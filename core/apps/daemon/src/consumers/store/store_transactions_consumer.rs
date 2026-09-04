use std::collections::HashSet;
use std::{collections::HashMap, error::Error};

use async_trait::async_trait;
use futures::{StreamExt, stream};
use primitives::{AssetIdVecExt, Chain, DeviceSubscription, NFTAssetId, NFTChain, Transaction, TransactionId, TransactionState, TransactionType};
use storage::{AssetFilter, AssetsAddressesRepository, AssetsRepository, Database, NftAssetFilter, NftRepository, TransactionsRepository, WalletsRepository};
use streamer::{
    AssetId, NotificationsPayload, StreamProducer, StreamProducerQueue, TransactionNotificationType, TransactionsPayload, WalletStreamEvent, WalletStreamPayload,
    consumer::MessageConsumer,
};
use swapper::cross_chain::{self, DepositAddressMap, SendAddressMap};

use crate::client::SwapVaultAddressClient;
use crate::consumers::store::StoreTransactionsConsumerConfig;
use crate::pusher::Pusher;

const TRANSACTION_BATCH_SIZE: usize = 100;

const CROSS_CHAIN_SOURCE_TYPES: [TransactionType; 3] = [TransactionType::Transfer, TransactionType::SmartContractCall, TransactionType::Swap];

pub struct StoreTransactionsConsumer {
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub pusher: Pusher,
    pub config: StoreTransactionsConsumerConfig,
    pub vault_client: SwapVaultAddressClient,
}

#[async_trait]
impl MessageConsumer<TransactionsPayload, usize> for StoreTransactionsConsumer {
    async fn should_process(&self, _payload: &TransactionsPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: TransactionsPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.chain;
        let is_notify_devices = payload.should_notify_devices();
        let deposit_addresses = self.vault_client.get_deposit_address_map().await?;
        let send_addresses = self.vault_client.get_send_address_map().await?;
        let transactions = Self::transactions_for_storage(payload.transactions, &deposit_addresses, &send_addresses)
            .into_iter()
            .filter(|transaction| self.config.is_transaction_within_asset_transfer_limit(transaction))
            .collect::<Vec<_>>();

        let min_amount = self.config.min_amount_usd;

        let addresses: Vec<_> = transactions
            .iter()
            .flat_map(|transaction| transaction.addresses())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let subscriptions = self.database.wallets()?.get_subscriptions_by_chain_addresses(chain, addresses)?;
        let notification_subscriptions = Self::unique_subscriptions_per_device(subscriptions.clone());

        let subscription_addresses: HashSet<_> = subscriptions.iter().map(|s| &s.address).collect();

        let asset_ids: Vec<AssetId> = transactions
            .iter()
            .filter(|x| x.addresses().iter().any(|addr| subscription_addresses.contains(addr)))
            .flat_map(|x| x.asset_ids())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let (existing_assets, missing_assets) = self.get_existing_and_missing_assets(asset_ids)?;
        let existing_assets_map: HashMap<AssetId, primitives::AssetPriceMetadata> = existing_assets.into_iter().map(|asset| (asset.asset.asset.id.clone(), asset)).collect();

        let _ = self.stream_producer.publish_fetch_assets(missing_assets).await;

        let nft_asset_ids: Vec<NFTAssetId> = transactions
            .iter()
            .filter(|x| x.addresses().iter().any(|addr| subscription_addresses.contains(addr)))
            .filter_map(|x| x.nft_asset_id())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let missing_nft_assets = self.get_missing_nft_assets(Self::supported_nft_asset_ids(nft_asset_ids))?;
        let _ = self.stream_producer.publish_fetch_nft_assets(missing_nft_assets).await;

        let subscribed_transactions = subscriptions
            .iter()
            .flat_map(|subscription| transactions.iter().map(move |transaction| (subscription, transaction)))
            .filter(|(subscription, transaction)| transaction.addresses().contains(&subscription.address))
            .filter(|(_, transaction)| transaction.asset_ids().iter().all(|id| existing_assets_map.contains_key(id)))
            .filter(|(subscription, transaction)| {
                let transaction = transaction.finalize(vec![subscription.address.clone()]);
                existing_assets_map.get(&transaction.asset_id).is_some_and(|asset_price| {
                    !self
                        .config
                        .is_transaction_insufficient_amount(&transaction, &asset_price.asset.asset, asset_price.price, min_amount)
                })
            })
            .collect::<Vec<_>>();
        let transactions_map = subscribed_transactions
            .iter()
            .map(|(_, transaction)| (transaction.id.clone(), (*transaction).clone()))
            .collect::<HashMap<_, _>>();
        let assets_addresses = subscribed_transactions
            .iter()
            .filter(|(_, transaction)| Self::should_store_asset_addresses(transaction))
            .flat_map(|(subscription, transaction)| {
                transaction
                    .assets_addresses_with_fee()
                    .into_iter()
                    .filter(|address| address.address == subscription.address)
                    .filter(|address| existing_assets_map.contains_key(&address.asset_id))
            })
            .collect::<HashSet<_>>();

        let transaction_count = transactions_map.len();
        let inserted_transaction_ids = self.upsert_transactions(transactions_map.values().cloned().collect())?;
        let publishable_transactions = transactions_map
            .values()
            .filter(|transaction| should_publish_transaction(&payload.notification_type, inserted_transaction_ids.contains(&transaction.id)))
            .collect::<Vec<_>>();

        let notification_requests = notification_subscriptions
            .iter()
            .flat_map(|subscription| {
                publishable_transactions.iter().filter_map(|transaction| {
                    if !transaction.addresses().contains(&subscription.address) || !self.config.should_notify_transaction(transaction, is_notify_devices, &send_addresses) {
                        return None;
                    }

                    let assets = transaction
                        .asset_ids()
                        .iter()
                        .filter_map(|id| existing_assets_map.get(id))
                        .map(|asset_price| asset_price.asset.asset.clone())
                        .collect();
                    Some((subscription.clone(), (**transaction).clone(), assets))
                })
            })
            .collect::<Vec<_>>();
        let notifications = stream::iter(notification_requests)
            .filter_map(|(subscription, transaction, assets)| async move {
                match self.pusher.get_messages(&subscription, transaction, assets).await {
                    Ok(messages) => Some(NotificationsPayload::new(messages)),
                    Err(_) => None,
                }
            })
            .collect::<Vec<_>>()
            .await;

        let wallet_events = subscriptions
            .iter()
            .map(|subscription| subscription.wallet_row_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .filter_map(|wallet_id| {
                let wallet_transactions = subscribed_transactions
                    .iter()
                    .filter(|(subscription, _)| subscription.wallet_row_id == wallet_id)
                    .filter(|(_, transaction)| publishable_transactions.iter().any(|candidate| candidate.id == transaction.id))
                    .map(|(_, transaction)| (transaction.id.clone(), *transaction))
                    .collect::<HashMap<_, _>>();
                (!wallet_transactions.is_empty()).then(|| WalletStreamPayload {
                    wallet_id,
                    event: WalletStreamEvent::Transactions {
                        transaction_ids: wallet_transactions.keys().cloned().collect(),
                        asset_ids: wallet_transactions
                            .values()
                            .flat_map(|transaction| transaction.asset_ids())
                            .collect::<HashSet<_>>()
                            .into_iter()
                            .collect(),
                    },
                })
            })
            .collect();

        self.database.assets_addresses()?.add_assets_addresses(assets_addresses.into_iter().collect())?;
        let _ = self.stream_producer.publish_notifications_transactions(notifications).await;
        let _ = self.stream_producer.publish_wallet_stream_events(wallet_events).await;

        Ok(transaction_count)
    }
}

impl StoreTransactionsConsumer {
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

    fn get_existing_and_missing_assets(&self, assets_ids: Vec<AssetId>) -> Result<(Vec<primitives::AssetPriceMetadata>, Vec<AssetId>), Box<dyn Error + Send + Sync>> {
        let assets_with_prices = self
            .database
            .assets()?
            .get_assets_with_prices(vec![AssetFilter::Ids(assets_ids.clone().ids())], self.config.primary_price_max_age)?;
        let existing_ids = assets_with_prices.iter().map(|asset| asset.asset.asset.id.clone()).collect::<HashSet<_>>();
        let missing_assets = assets_ids.into_iter().filter(|asset_id| !existing_ids.contains(asset_id)).collect();
        let enabled_assets = assets_with_prices.into_iter().filter(|asset| asset.asset.properties.is_enabled).collect();
        Ok((enabled_assets, missing_assets))
    }

    fn get_missing_nft_assets(&self, nft_asset_ids: Vec<NFTAssetId>) -> Result<Vec<NFTAssetId>, Box<dyn Error + Send + Sync>> {
        if nft_asset_ids.is_empty() {
            return Ok(Vec::new());
        }
        let identifiers: Vec<String> = nft_asset_ids.iter().map(|id| id.to_string()).collect();
        let existing = self.database.nft()?.get_nft_assets_by_filter(vec![NftAssetFilter::Identifiers(identifiers)])?;
        let existing_ids: HashSet<NFTAssetId> = existing.into_iter().map(|row| row.identifier.0).collect();
        Ok(nft_asset_ids.into_iter().filter(|id| !existing_ids.contains(id)).collect())
    }

    fn upsert_transactions(&self, transactions: Vec<Transaction>) -> Result<HashSet<TransactionId>, Box<dyn Error + Send + Sync>> {
        transactions
            .chunks(TRANSACTION_BATCH_SIZE)
            .try_fold(HashSet::new(), |inserted_ids, chunk| -> Result<HashSet<TransactionId>, Box<dyn Error + Send + Sync>> {
                let chunk_inserted_ids = self.database.transactions()?.upsert_transactions(chunk.to_vec())?;
                Ok(inserted_ids.into_iter().chain(chunk_inserted_ids).collect())
            })
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
    use num_bigint::BigUint;
    use primitives::{AssetId, Device, SwapProvider, TransactionSwapMetadata, WalletId, contract_constants::SOLANA_RELAY_DEPOSITORY_PROGRAM_ID};

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
                serde_json::to_value(TransactionSwapMetadata {
                    from_asset: AssetId::from_chain(Chain::Solana),
                    from_value: BigUint::from(5000000u64),
                    to_asset: AssetId::from_chain(Chain::Ton),
                    to_value: BigUint::from(2508437099u64),
                    provider: Some(SwapProvider::NearIntents.as_ref().to_string()),
                })
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

        let near_intents = Transaction {
            to: near_vault.clone(),
            ..Transaction::mock()
        };
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
            from: thorchain_vault.clone(),
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
        let result = StoreTransactionsConsumer::unique_subscriptions_per_device(vec![multicoin.clone(), other_device.clone()]);
        assert_eq!(result.len(), 2);
    }
}
