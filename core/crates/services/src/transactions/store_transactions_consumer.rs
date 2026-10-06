use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use futures::{StreamExt, TryStreamExt, stream};
use primitives::{AssetAddress, AssetIdVecExt, AssetPriceMetadata, Chain, DeviceSubscription, NFTAssetId, NFTChain, Transaction, TransactionId, TransactionState, TransactionType};
use storage::AssetFilter;
use streamer::{AssetId, NotificationsPayload, QueueName, StreamProducerQueue, TransactionNotificationType, TransactionsPayload, WalletStreamEvent, WalletStreamPayload, consumer::MessageConsumer};
use swapper::cross_chain::{self, DepositAddressMap, SendAddressMap};

use super::StoreTransactionsConsumerConfig;
use super::SwapVaultAddressClient;
use super::repository::Repository;
use crate::config::ConfigCacher;
use crate::notifications::Pusher;
use crate::subscriptions::SubscriptionLookup;
use push_notification::GorushNotification;

const CROSS_CHAIN_SOURCE_TYPES: [TransactionType; 3] = [TransactionType::Transfer, TransactionType::SmartContractCall, TransactionType::Swap];

pub struct StoreTransactionsConsumer {
    pub(crate) repository: Arc<dyn Repository>,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
    pub pusher: Pusher,
    pub config: Arc<ConfigCacher>,
    pub vault_client: SwapVaultAddressClient,
    pub(crate) subscription_lookup: Arc<SubscriptionLookup>,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for StoreTransactionsConsumer {
    async fn should_consume(&self, _payload: &TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let config = StoreTransactionsConsumerConfig::read(&self.config).await?;
        let referral_transactions = payload.transactions.iter().filter(|transaction| referral_queue(transaction).is_some()).cloned().collect();
        let count = self.store_subscribed_transactions(&config, payload).await?;
        self.store_referral_transactions(&config, referral_transactions).await?;
        Ok(count)
    }
}

impl StoreTransactionsConsumer {
    async fn store_subscribed_transactions(&self, config: &StoreTransactionsConsumerConfig, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let is_notify_devices = payload.should_notify_devices();
        let subscriptions = self.get_subscriptions(payload.chain, &payload.transactions).await?;
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let (deposit_addresses, send_addresses) = tokio::try_join!(self.vault_client.get_deposit_address_map(), self.vault_client.get_send_address_map())?;
        let transactions = Self::subscribed_transactions_for_storage(config, payload.transactions, &subscriptions, &deposit_addresses, &send_addresses);
        if transactions.is_empty() {
            return Ok(0);
        }

        let assets = self.get_existing_assets_and_fetch_missing(config, &transactions).await?;
        let subscribed_transactions = Self::subscribed_transactions(config, &subscriptions, &transactions, &assets);
        let transactions_map = subscribed_transactions.iter().map(|(_, transaction)| (transaction.id.clone(), (*transaction).clone())).collect::<HashMap<_, _>>();
        if transactions_map.is_empty() {
            return Ok(0);
        }
        let assets_addresses = Self::assets_addresses(&subscribed_transactions, &assets);

        let transaction_count = transactions_map.len();
        let inserted_transaction_ids = self.upsert_transactions(transactions_map.values().cloned().collect(), config.batch_size).await?;
        let publishable_transactions = transactions_map
            .values()
            .filter(|transaction| should_publish_transaction(&payload.notification_type, inserted_transaction_ids.contains(&transaction.id)))
            .collect::<Vec<_>>();

        let notifications = self.get_notifications(config, &subscriptions, &publishable_transactions, &assets, is_notify_devices, &send_addresses).await?;
        let wallet_events = Self::wallet_events(&subscriptions, &subscribed_transactions, &publishable_transactions);

        if !assets_addresses.is_empty() {
            self.repository.add_asset_addresses(assets_addresses).await?;
        }
        self.stream_producer.publish_notifications_transactions(notifications).await?;
        self.stream_producer.publish_wallet_stream_events(wallet_events).await?;

        Ok(transaction_count)
    }

    async fn store_referral_transactions(&self, config: &StoreTransactionsConsumerConfig, transactions: Vec<Transaction>) -> Result<(), Box<dyn Error + Send + Sync>> {
        if transactions.is_empty() {
            return Ok(());
        }
        let (deposit_addresses, send_addresses) = tokio::try_join!(self.vault_client.get_deposit_address_map(), self.vault_client.get_send_address_map())?;
        let transactions = Self::transactions_for_storage(transactions, &deposit_addresses, &send_addresses);
        let asset_ids: Vec<AssetId> = transactions.iter().flat_map(Transaction::asset_ids).collect::<HashSet<_>>().into_iter().collect();
        let existing_ids = self.repository.assets(asset_ids.clone()).await?.into_iter().map(|asset| asset.id).collect::<HashSet<_>>();
        self.stream_producer.publish_fetch_assets(asset_ids.into_iter().filter(|id| !existing_ids.contains(id)).collect()).await?;

        let transactions = transactions.into_iter().filter(|transaction| transaction.asset_ids().iter().all(|id| existing_ids.contains(id))).collect::<Vec<_>>();
        let referrals = transactions.iter().filter_map(|transaction| Some((referral_queue(transaction)?, transaction.id.clone()))).collect::<Vec<_>>();
        self.upsert_transactions(transactions, config.batch_size).await?;
        for (queue, transaction_id) in referrals {
            self.stream_producer.publish_referral_transaction(queue, transaction_id).await?;
        }
        Ok(())
    }

    async fn get_subscriptions(&self, chain: Chain, transactions: &[Transaction]) -> Result<Vec<DeviceSubscription>, Box<dyn Error + Send + Sync>> {
        let addresses = transactions.iter().flat_map(Transaction::addresses).collect();
        self.subscription_lookup.get(chain, addresses).await
    }

    fn subscribed_transactions_for_storage(
        config: &StoreTransactionsConsumerConfig,
        transactions: Vec<Transaction>,
        subscriptions: &[DeviceSubscription],
        deposit_addresses: &DepositAddressMap,
        send_addresses: &SendAddressMap,
    ) -> Vec<Transaction> {
        let subscription_addresses: HashSet<_> = subscriptions.iter().map(|subscription| &subscription.address).collect();
        Self::transactions_for_storage(transactions, deposit_addresses, send_addresses)
            .into_iter()
            .filter(|transaction| config.is_transaction_within_asset_transfer_limit(transaction))
            .filter(|transaction| transaction.addresses().iter().any(|address| subscription_addresses.contains(address)))
            .collect()
    }

    async fn get_existing_assets_and_fetch_missing(&self, config: &StoreTransactionsConsumerConfig, transactions: &[Transaction]) -> Result<HashMap<AssetId, AssetPriceMetadata>, Box<dyn Error + Send + Sync>> {
        let asset_ids: Vec<AssetId> = transactions.iter().flat_map(Transaction::asset_ids).collect::<HashSet<_>>().into_iter().collect();
        let nft_asset_ids: Vec<NFTAssetId> = transactions.iter().filter_map(Transaction::nft_asset_id).collect::<HashSet<_>>().into_iter().collect();

        let ((existing_assets, missing_assets), missing_nft_assets) = tokio::try_join!(
            self.get_existing_and_missing_assets(asset_ids, config.primary_price_max_age),
            self.get_missing_nft_assets(Self::supported_nft_asset_ids(nft_asset_ids)),
        )?;

        self.stream_producer.publish_fetch_assets(missing_assets).await?;
        self.stream_producer.publish_fetch_nft_assets(missing_nft_assets).await?;

        Ok(existing_assets.into_iter().map(|asset| (asset.asset.asset.id.clone(), asset)).collect())
    }

    fn subscribed_transactions<'a>(
        config: &StoreTransactionsConsumerConfig,
        subscriptions: &'a [DeviceSubscription],
        transactions: &'a [Transaction],
        assets: &HashMap<AssetId, AssetPriceMetadata>,
    ) -> Vec<(&'a DeviceSubscription, &'a Transaction)> {
        subscriptions
            .iter()
            .flat_map(|subscription| transactions.iter().map(move |transaction| (subscription, transaction)))
            .filter(|(subscription, transaction)| transaction.addresses().contains(&subscription.address))
            .filter(|(_, transaction)| transaction.asset_ids().iter().all(|id| assets.get(id).is_some_and(|asset| asset.asset.is_enabled_for_transactions())))
            .filter(|(subscription, transaction)| {
                let transaction = transaction.finalize(vec![subscription.address.clone()]);
                assets
                    .get(&transaction.asset_id)
                    .is_some_and(|asset_price| !config.is_transaction_insufficient_amount(&transaction, &asset_price.asset.asset, asset_price.price, config.min_amount_usd))
            })
            .collect()
    }

    fn assets_addresses(subscribed_transactions: &[(&DeviceSubscription, &Transaction)], assets: &HashMap<AssetId, AssetPriceMetadata>) -> Vec<AssetAddress> {
        subscribed_transactions
            .iter()
            .filter(|(_, transaction)| Self::should_store_asset_addresses(transaction))
            .flat_map(|(subscription, transaction)| {
                transaction
                    .assets_addresses_with_fee()
                    .into_iter()
                    .filter(|address| address.address == subscription.address)
                    .filter(|address| assets.contains_key(&address.asset_id))
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    async fn get_notifications(
        &self,
        config: &StoreTransactionsConsumerConfig,
        subscriptions: &[DeviceSubscription],
        publishable_transactions: &[&Transaction],
        assets: &HashMap<AssetId, AssetPriceMetadata>,
        is_notify_devices: bool,
        send_addresses: &SendAddressMap,
    ) -> Result<Vec<NotificationsPayload>, Box<dyn Error + Send + Sync>> {
        let notification_requests = Self::unique_subscriptions_per_device(subscriptions.to_vec())
            .into_iter()
            .flat_map(|subscription| {
                publishable_transactions
                    .iter()
                    .filter(|transaction| transaction.addresses().contains(&subscription.address) && config.should_notify_transaction(transaction, &subscription.address, is_notify_devices, send_addresses))
                    .map(|transaction| {
                        let assets = transaction.asset_ids().iter().filter_map(|id| assets.get(id)).map(|asset_price| asset_price.asset.asset.clone()).collect();
                        (subscription.clone(), (*transaction).clone(), assets)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let notifications: Vec<Vec<GorushNotification>> = stream::iter(notification_requests)
            .then(|(subscription, transaction, assets)| async move { self.pusher.get_messages(&subscription, transaction, assets).await })
            .try_collect()
            .await?;
        Ok(NotificationsPayload::batches(notifications.into_iter().flatten().collect(), config.notifications_batch_size))
    }

    fn wallet_events(subscriptions: &[DeviceSubscription], subscribed_transactions: &[(&DeviceSubscription, &Transaction)], publishable_transactions: &[&Transaction]) -> Vec<WalletStreamPayload> {
        subscriptions
            .iter()
            .map(|subscription| subscription.wallet_row_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .flat_map(|wallet_id| {
                let wallet_transactions = subscribed_transactions
                    .iter()
                    .filter(|(subscription, _)| subscription.wallet_row_id == wallet_id)
                    .filter(|(_, transaction)| publishable_transactions.iter().any(|candidate| candidate.id == transaction.id))
                    .map(|(_, transaction)| (transaction.id.clone(), *transaction))
                    .collect::<HashMap<_, _>>();
                let transactions = (!wallet_transactions.is_empty()).then(|| WalletStreamPayload {
                    wallet_id,
                    event: WalletStreamEvent::Transactions {
                        transaction_ids: wallet_transactions.keys().cloned().collect(),
                        asset_ids: wallet_transactions.values().flat_map(|transaction| transaction.asset_ids()).collect::<HashSet<_>>().into_iter().collect(),
                    },
                });
                let nfts = wallet_transactions
                    .values()
                    .any(|transaction| transaction.transaction_type == TransactionType::TransferNFT)
                    .then_some(WalletStreamPayload { wallet_id, event: WalletStreamEvent::Nft });
                transactions.into_iter().chain(nfts)
            })
            .collect()
    }

    fn supported_nft_asset_ids(nft_asset_ids: Vec<NFTAssetId>) -> Vec<NFTAssetId> {
        let supported_chains = NFTChain::all().into_iter().map(Chain::from).collect::<HashSet<_>>();
        nft_asset_ids.into_iter().filter(|asset_id| supported_chains.contains(&asset_id.chain)).collect()
    }

    fn should_store_asset_addresses(transaction: &Transaction) -> bool {
        match transaction.state {
            TransactionState::Confirmed | TransactionState::InTransit => true,
            TransactionState::Pending | TransactionState::Failed | TransactionState::Reverted | TransactionState::Refunded => false,
        }
    }

    fn unique_subscriptions_per_device(subscriptions: Vec<DeviceSubscription>) -> Vec<DeviceSubscription> {
        subscriptions
            .into_iter()
            .fold(HashMap::<(String, String), DeviceSubscription>::new(), |mut best, sub| {
                let key = (sub.device.id.clone(), sub.address.clone());
                best.entry(key)
                    .and_modify(|existing| {
                        if sub.wallet_id.wallet_type().rank() < existing.wallet_id.wallet_type().rank() {
                            *existing = sub.clone();
                        }
                    })
                    .or_insert(sub);
                best
            })
            .into_values()
            .collect()
    }

    fn transactions_for_storage(transactions: Vec<Transaction>, deposit_addresses: &DepositAddressMap, send_addresses: &SendAddressMap) -> Vec<Transaction> {
        transactions
            .into_iter()
            .filter_map(|mut transaction| {
                if cross_chain::is_from_vault_address(&transaction, send_addresses) {
                    return None;
                }

                if Self::should_mark_in_transit(&transaction, deposit_addresses) {
                    transaction.state = TransactionState::InTransit;
                }

                Some(transaction)
            })
            .collect()
    }

    fn should_mark_in_transit(transaction: &Transaction, deposit_addresses: &DepositAddressMap) -> bool {
        transaction.state == TransactionState::Confirmed
            && CROSS_CHAIN_SOURCE_TYPES.contains(&transaction.transaction_type)
            && !(transaction.transaction_type == TransactionType::Swap && transaction.metadata.is_some())
            && cross_chain::is_cross_chain_swap(transaction, deposit_addresses)
    }

    async fn get_existing_and_missing_assets(&self, assets_ids: Vec<AssetId>, primary_price_max_age: Duration) -> Result<(Vec<primitives::AssetPriceMetadata>, Vec<AssetId>), Box<dyn Error + Send + Sync>> {
        let filters = vec![AssetFilter::Ids(assets_ids.clone().ids())];
        let assets_with_prices = self.repository.assets_with_prices(filters, primary_price_max_age).await?;
        let existing_ids = assets_with_prices.iter().map(|asset| asset.asset.asset.id.clone()).collect::<HashSet<_>>();
        let missing_assets = assets_ids.into_iter().filter(|asset_id| !existing_ids.contains(asset_id)).collect();
        Ok((assets_with_prices, missing_assets))
    }

    async fn get_missing_nft_assets(&self, nft_asset_ids: Vec<NFTAssetId>) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        if nft_asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let identifiers: Vec<String> = nft_asset_ids.iter().map(ToString::to_string).collect();
        let existing_ids: HashSet<NFTAssetId> = self.repository.nft_asset_ids(identifiers).await?.into_iter().collect();
        Ok(nft_asset_ids.into_iter().filter(|id| !existing_ids.contains(id)).collect())
    }

    async fn upsert_transactions(&self, transactions: Vec<Transaction>, batch_size: usize) -> Result<HashSet<TransactionId>, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.upsert_transactions(transactions, batch_size).await?)
    }
}

fn referral_queue(transaction: &Transaction) -> Option<QueueName> {
    match transaction.transaction_type {
        TransactionType::Swap => transaction.swap_metadata()?.referral_fee.map(|_| QueueName::StoreTransactionsSwaps),
        TransactionType::PerpetualOpenPosition | TransactionType::PerpetualClosePosition => transaction.perpetual_metadata()?.referral_fee.map(|_| QueueName::StoreTransactionsPerpetuals),
        TransactionType::Transfer
        | TransactionType::TransferNFT
        | TransactionType::TokenApproval
        | TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeRedelegate
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze
        | TransactionType::AssetActivation
        | TransactionType::SmartContractCall
        | TransactionType::PerpetualModifyPosition
        | TransactionType::EarnDeposit
        | TransactionType::EarnWithdraw => None,
    }
}

fn should_publish_transaction(notification_type: &TransactionNotificationType, is_inserted: bool) -> bool {
    match notification_type {
        TransactionNotificationType::NewTransaction => is_inserted,
        TransactionNotificationType::StateChange => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_solana::{
        models::{BlockTransaction, SingleTransaction},
        provider::transaction_mapper::map_transaction,
    };
    use num_bigint::BigUint;
    use primitives::{
        Asset, AssetId, Device, JsonRpcResult, SwapProvider, TransactionPerpetualMetadata, TransactionSwapMetadata, TransactionSwapReferralFee, WalletId,
        asset_constants::SOLANA_USDC_ASSET_ID,
        contract_constants::{SOLANA_MAYAN_CPI_PROXY_PROGRAM_ID, SOLANA_RELAY_DEPOSITORY_PROGRAM_ID},
        known_assets::HYPERCORE_PERPETUAL_USDC,
    };

    #[test]
    fn test_mayan_swift_deposit_enters_cross_chain_processing() {
        let response: JsonRpcResult<SingleTransaction> = serde_json::from_str(include_str!("../../../gem_solana/testdata/mayan_swift_deposit_token.json")).unwrap();
        let source = BlockTransaction {
            meta: response.result.meta,
            transaction: response.result.transaction,
        };
        let transaction = map_transaction(&source, response.result.block_time).unwrap();
        let deposit_addresses = DepositAddressMap::from([(SOLANA_MAYAN_CPI_PROXY_PROGRAM_ID.to_string(), SwapProvider::Mayan)]);
        let transactions = StoreTransactionsConsumer::transactions_for_storage(vec![transaction], &deposit_addresses, &SendAddressMap::new());

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::InTransit);
        assert_eq!(transaction.asset_id, AssetId::from_token(Chain::Solana, "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"));
        assert_eq!(transaction.value, BigUint::from(1_000_000_000u64));
        assert_eq!(transaction.metadata, None);
        assert_eq!(cross_chain::swap_provider_with_vault_addresses(transaction, &deposit_addresses), Some(SwapProvider::Mayan));
    }

    #[test]
    fn test_relay_lookup_table_deposit_enters_cross_chain_processing() {
        let response: JsonRpcResult<SingleTransaction> = serde_json::from_str(include_str!("../../../gem_solana/testdata/relay_deposit_token_lookup_table.json")).unwrap();
        let source = BlockTransaction {
            meta: response.result.meta,
            transaction: response.result.transaction,
        };
        let transaction = map_transaction(&source, response.result.block_time).unwrap();
        let deposit_addresses = DepositAddressMap::from([(SOLANA_RELAY_DEPOSITORY_PROGRAM_ID.to_string(), SwapProvider::Relay)]);
        let transactions = StoreTransactionsConsumer::transactions_for_storage(vec![transaction], &deposit_addresses, &SendAddressMap::new());

        assert_eq!(transactions.len(), 1);
        let transaction = &transactions[0];
        assert_eq!(transaction.transaction_type, TransactionType::Swap);
        assert_eq!(transaction.state, TransactionState::InTransit);
        assert_eq!(transaction.asset_id, SOLANA_USDC_ASSET_ID.clone());
        assert_eq!(transaction.value, BigUint::from(5_000_000u64));
        assert_eq!(transaction.metadata, None);
        assert_eq!(cross_chain::swap_provider_with_vault_addresses(transaction, &deposit_addresses), Some(SwapProvider::Relay));
    }

    #[test]
    fn test_subscribed_transactions_asset_filter() {
        let config = StoreTransactionsConsumerConfig::mock();
        let subscriptions = vec![DeviceSubscription {
            address: "0xfrom".to_string(),
            ..DeviceSubscription::mock()
        }];
        let asset = |asset: Asset, is_enabled: bool| {
            let mut basic = asset.as_basic_primitive();
            basic.properties.is_enabled = is_enabled;
            (basic.asset.id.clone(), AssetPriceMetadata { asset: basic, price: None })
        };
        let assets = HashMap::from([asset(Asset::mock_eth(), true), asset(Asset::mock_ethereum_usdc(), false), asset(HYPERCORE_PERPETUAL_USDC.clone(), false)]);
        let transfer = Transaction::mock();
        let disabled_token_transfer = Transaction::mock_with_params(Asset::mock_ethereum_usdc().id, TransactionType::Transfer, BigUint::from(1u32));
        let perpetual = Transaction::mock_with_params(HYPERCORE_PERPETUAL_USDC.id.clone(), TransactionType::PerpetualOpenPosition, BigUint::from(1u32));
        let missing_asset_transfer = Transaction::mock_with_params(AssetId::from_chain(Chain::Bitcoin), TransactionType::Transfer, BigUint::from(1u32));
        let transactions = vec![transfer, disabled_token_transfer, perpetual, missing_asset_transfer];

        let subscribed = StoreTransactionsConsumer::subscribed_transactions(&config, &subscriptions, &transactions, &assets);

        assert_eq!(
            subscribed.iter().map(|(_, transaction)| transaction.asset_id.clone()).collect::<Vec<_>>(),
            vec![AssetId::from_chain(Chain::Ethereum), HYPERCORE_PERPETUAL_USDC.id.clone()]
        );
    }

    #[test]
    fn test_supported_nft_asset_ids() {
        let ethereum = NFTAssetId::mock();
        let base = NFTAssetId::new(Chain::Base, "0x1", "1");

        assert_eq!(StoreTransactionsConsumer::supported_nft_asset_ids(vec![ethereum.clone(), base]), vec![ethereum]);
    }

    #[test]
    fn test_should_publish_transaction() {
        assert!(should_publish_transaction(&TransactionNotificationType::NewTransaction, true));
        assert!(!should_publish_transaction(&TransactionNotificationType::NewTransaction, false));
        assert!(should_publish_transaction(&TransactionNotificationType::StateChange, false));
    }

    #[test]
    fn test_referral_queue() {
        let referral_fee = TransactionSwapReferralFee {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            value: BigUint::from(1u32),
        };
        let referral_swap = Transaction {
            metadata: serde_json::to_value(TransactionSwapMetadata::mock().with_referral_fee(Some(referral_fee.clone()))).ok(),
            ..Transaction::mock_swap()
        };
        let referral_perpetual = Transaction {
            transaction_type: TransactionType::PerpetualOpenPosition,
            metadata: serde_json::to_value(TransactionPerpetualMetadata {
                referral_fee: Some(referral_fee),
                ..TransactionPerpetualMetadata::mock()
            })
            .ok(),
            ..Transaction::mock()
        };

        assert_eq!(referral_queue(&referral_swap), Some(QueueName::StoreTransactionsSwaps));
        assert_eq!(referral_queue(&referral_perpetual), Some(QueueName::StoreTransactionsPerpetuals));
        assert_eq!(referral_queue(&Transaction::mock_swap()), None);
        assert_eq!(
            referral_queue(&Transaction {
                transaction_type: TransactionType::PerpetualOpenPosition,
                metadata: serde_json::to_value(TransactionPerpetualMetadata::mock()).ok(),
                ..Transaction::mock()
            }),
            None
        );
        assert_eq!(
            referral_queue(&Transaction {
                transaction_type: TransactionType::Transfer,
                ..referral_swap
            }),
            None
        );
    }

    #[test]
    fn test_transactions_for_storage() {
        let thorchain_vault = "0xD37BbE5744D730a1d98d8DC97c42F0Ca46aD7146".to_string();
        let near_vault = "TMoD2uJiUAvB2RhLGm1BmzCVVzi5VLFDVt".to_string();
        let relay_depository = SOLANA_RELAY_DEPOSITORY_PROGRAM_ID.to_string();
        let deposit_addresses = DepositAddressMap::from([
            (thorchain_vault.clone(), SwapProvider::Thorchain),
            (near_vault.clone(), SwapProvider::NearIntents),
            (relay_depository.clone(), SwapProvider::Relay),
        ]);
        let send_addresses = SendAddressMap::from([(thorchain_vault.clone(), SwapProvider::Thorchain), (near_vault.clone(), SwapProvider::NearIntents)]);

        let cross_chain = Transaction {
            to: thorchain_vault.clone(),
            memo: Some("=:BTC:bc1qaddress:0/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![cross_chain], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let vault_no_memo = Transaction {
            to: "bc1qvault".to_string(),
            ..Transaction::mock()
        };
        let deposit_addresses_bc = DepositAddressMap::from([("bc1qvault".to_string(), SwapProvider::Thorchain)]);
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![vault_no_memo], &deposit_addresses_bc, &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![Transaction::mock()], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let swap_type = Transaction {
            transaction_type: TransactionType::Swap,
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![swap_type], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let cross_chain_swap_type = Transaction {
            transaction_type: TransactionType::Swap,
            to: near_vault.clone(),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![cross_chain_swap_type], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let confirmed_cross_chain_swap_update = Transaction {
            transaction_type: TransactionType::Swap,
            to: near_vault.clone(),
            metadata: Some(
                serde_json::to_value(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Solana),
                    BigUint::from(5000000u64),
                    AssetId::from_chain(Chain::Ton),
                    BigUint::from(2508437099u64),
                    SwapProvider::NearIntents,
                ))
                .unwrap(),
            ),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![confirmed_cross_chain_swap_update], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let token_approval = Transaction {
            transaction_type: TransactionType::TokenApproval,
            to: "0x337685fdaB40D39bd02028545a4FfA7D287cC3E2".to_string(),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![token_approval], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Confirmed
        );

        let pending = Transaction {
            state: TransactionState::Pending,
            memo: Some("=:ETH.USDT:0x858734a6353C9921a78fB3c937c8E20Ba6f36902:1635978e6/1/0".to_string()),
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![pending], &DepositAddressMap::new(), &SendAddressMap::new())[0].state,
            TransactionState::Pending
        );

        let near_intents = Transaction { to: near_vault, ..Transaction::mock() };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![near_intents], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let relay = Transaction {
            transaction_type: TransactionType::Swap,
            to: relay_depository,
            ..Transaction::mock()
        };
        assert_eq!(
            StoreTransactionsConsumer::transactions_for_storage(vec![relay], &deposit_addresses, &SendAddressMap::new())[0].state,
            TransactionState::InTransit
        );

        let outbound = Transaction {
            from: thorchain_vault,
            ..Transaction::mock()
        };
        let regular = Transaction::mock();

        let transactions = StoreTransactionsConsumer::transactions_for_storage(vec![outbound, regular.clone()], &DepositAddressMap::new(), &send_addresses);

        assert_eq!(transactions, vec![regular]);
    }

    #[test]
    fn test_should_store_asset_addresses() {
        assert!(StoreTransactionsConsumer::should_store_asset_addresses(&Transaction::mock()));
        assert!(StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::InTransit,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Failed,
            ..Transaction::mock()
        }));
        assert!(!StoreTransactionsConsumer::should_store_asset_addresses(&Transaction {
            state: TransactionState::Reverted,
            ..Transaction::mock()
        }));
    }

    #[test]
    fn test_unique_subscriptions_per_device() {
        let multicoin = DeviceSubscription::mock();
        let single = DeviceSubscription {
            wallet_id: WalletId::Single(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };
        let view = DeviceSubscription {
            wallet_id: WalletId::View(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone(), single.clone(), multicoin.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, multicoin.wallet_id);

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone(), single.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, single.wallet_id);

        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![view.clone()]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].wallet_id, view.wallet_id);

        let other_device = DeviceSubscription {
            device: Device {
                id: "device-2".to_string(),
                ..Device::mock()
            },
            wallet_id: WalletId::View(Chain::Ethereum, "0xABC".to_string()),
            ..DeviceSubscription::mock()
        };
        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![multicoin, other_device]);
        assert_eq!(result.len(), 2);
    }
}
