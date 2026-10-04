use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;
use std::sync::Arc;

use primitives::{AddressChains, Chain, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};
use storage::{Database, DatabaseError, DevicesRepository, FiatRepository, NewWallet, NftRepository, RewardsRepository, TransactionsRepository, WalletsRepository};
use streamer::{ChainAddressPayload, StreamProducerQueue};

use super::admin_device::AdminWalletOverview;

#[derive(Clone)]
pub struct WalletsClient {
    database: Database,
    stream_producer: Arc<dyn StreamProducerQueue>,
}

impl WalletsClient {
    pub fn new(database: Database, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { database, stream_producer }
    }

    pub async fn get_subscriptions(&self, device_row_id: i32) -> Result<Vec<WalletSubscriptionChains>, Box<dyn Error + Send + Sync>> {
        let rows = self.database.run(move |client| client.get_subscriptions(device_row_id)).await?;

        Ok(rows
            .into_iter()
            .fold(BTreeMap::<String, (WalletId, Vec<Chain>)>::new(), |mut acc, (wallet, subscription)| {
                acc.entry(wallet.wallet_id.id()).or_insert((wallet.wallet_id, Vec::new())).1.push(subscription.chain);
                acc
            })
            .into_values()
            .map(|(wallet_id, mut chains)| {
                chains.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
                WalletSubscriptionChains { wallet_id, chains }
            })
            .collect())
    }

    pub async fn get_wallet_subscriptions(&self, device_id: &str) -> Result<Vec<WalletSubscription>, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let rows = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let device_row_id = client.get_device_row_id(&device_id)?;
                client.get_subscriptions(device_row_id)
            })
            .await?;
        let mut subscriptions = BTreeMap::<String, (WalletId, WalletSource, BTreeMap<String, BTreeSet<Chain>>)>::new();

        for (wallet, subscription) in rows {
            subscriptions
                .entry(wallet.wallet_id.id())
                .or_insert_with(|| (wallet.wallet_id, wallet.source, BTreeMap::new()))
                .2
                .entry(subscription.address)
                .or_default()
                .insert(subscription.chain);
        }

        Ok(subscriptions.into_values().map(|(wallet_id, source, addresses)| wallet_subscription(wallet_id, source, addresses)).collect())
    }

    pub async fn get_wallet_overviews(&self, device_row_id: i32) -> Result<Vec<AdminWalletOverview>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .run(move |client| -> Result<Vec<AdminWalletOverview>, DatabaseError> {
                let rows = client.get_subscriptions(device_row_id)?;

                rows.into_iter()
                    .fold(BTreeMap::<String, WalletOverviewBuilder>::new(), |mut wallets, (wallet, subscription)| {
                        let entry = wallets.entry(wallet.wallet_id.id()).or_insert_with(|| WalletOverviewBuilder {
                            wallet_id: wallet.id,
                            identifier: wallet.wallet_id,
                            source: wallet.source,
                            addresses: BTreeSet::new(),
                            chains: BTreeSet::new(),
                            subscription_count: 0,
                        });
                        entry.subscription_count += 1;
                        entry.addresses.insert(subscription.address);
                        entry.chains.insert(subscription.chain);
                        wallets
                    })
                    .into_values()
                    .map(|wallet| {
                        let chains = wallet.chains.into_iter().collect::<Vec<_>>();
                        let addresses = wallet.addresses.into_iter().collect::<Vec<_>>();
                        Ok(AdminWalletOverview {
                            transaction_count: client.count_transactions_by_addresses(addresses.clone(), chains.iter().map(|chain| chain.as_ref().to_string()).collect())?,
                            fiat_transaction_count: client.count_fiat_transactions_by_device_and_wallet_id(device_row_id, wallet.wallet_id)?,
                            nft_count: client.count_nft_assets_by_addresses(addresses, chains.clone())?,
                            chains,
                            id: wallet.identifier,
                            source: wallet.source,
                            username: client.get_username_by_wallet_id(wallet.wallet_id)?,
                            subscription_count: wallet.subscription_count,
                        })
                    })
                    .collect()
            })
            .await?)
    }

    pub async fn get_wallet_subscription(&self, device_id: &str, wallet_id: &str) -> Result<WalletSubscription, Box<dyn Error + Send + Sync>> {
        let device_id = device_id.to_string();
        let wallet_id = wallet_id.to_string();
        let (wallet, rows) = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let device_row_id = client.get_device_row_id(&device_id)?;
                let wallet = client.get_wallet_by_device_and_identifier(device_row_id, &wallet_id)?;
                let rows = client.get_subscriptions_by_wallet_id(device_row_id, wallet.id)?;
                Ok((wallet, rows))
            })
            .await?;
        let mut addresses = BTreeMap::<String, BTreeSet<Chain>>::new();

        for subscription in rows {
            addresses.entry(subscription.address).or_default().insert(subscription.chain);
        }

        Ok(wallet_subscription(wallet.wallet_id, wallet.source, addresses))
    }

    pub async fn add_subscriptions(&self, device_row_id: i32, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if wallet_subscriptions.is_empty() {
            return Ok(0);
        }

        let payload: Vec<ChainAddressPayload> = wallet_subscriptions
            .iter()
            .filter(|x| x.source == Some(WalletSource::Import))
            .flat_map(WalletSubscription::chain_addresses)
            .map(ChainAddressPayload::from)
            .collect();
        let count = self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
                let mut wallet_ids: HashMap<String, i32> = client.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

                let new_wallets: Vec<NewWallet> = wallet_subscriptions
                    .iter()
                    .filter(|x| !wallet_ids.contains_key(&x.wallet_id.id()))
                    .map(|x| NewWallet {
                        wallet_id: x.wallet_id.clone(),
                        wallet_type: x.wallet_id.wallet_type(),
                        source: x.source.clone().unwrap_or(WalletSource::Import),
                    })
                    .collect();

                if !new_wallets.is_empty() {
                    let new_identifiers: Vec<String> = new_wallets.iter().map(|x| x.wallet_id.id()).collect();
                    client.create_wallets(new_wallets)?;
                    wallet_ids.extend(client.get_wallets(new_identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)));
                }

                let subscriptions: Vec<(i32, Chain, String)> = wallet_subscriptions
                    .iter()
                    .filter_map(|ws| wallet_ids.get(&ws.wallet_id.id()).map(|&wallet_id| ws.chain_addresses().into_iter().map(move |ca| (wallet_id, ca.chain, ca.address))))
                    .flatten()
                    .collect();

                client.add_subscriptions(device_row_id, subscriptions)
            })
            .await?;

        if !payload.is_empty() {
            self.stream_producer.publish_new_addresses(payload).await?;
        }

        Ok(count)
    }

    pub async fn delete_subscriptions(&self, device_row_id: i32, subscriptions: Vec<WalletSubscriptionChains>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        Ok(self
            .database
            .run(move |client| -> Result<_, DatabaseError> {
                let identifiers: Vec<String> = subscriptions.iter().map(|x| x.wallet_id.id()).collect();
                let wallet_ids: HashMap<String, i32> = client.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

                let mut count = 0;
                for ws in subscriptions {
                    if let Some(&wallet_id) = wallet_ids.get(&ws.wallet_id.id()) {
                        count += client.delete_wallet_chains(device_row_id, wallet_id, ws.chains)?;
                    }
                }

                Ok(count)
            })
            .await?)
    }
}

fn wallet_subscription(wallet_id: WalletId, source: WalletSource, addresses: BTreeMap<String, BTreeSet<Chain>>) -> WalletSubscription {
    WalletSubscription {
        wallet_id,
        source: Some(source),
        subscriptions: addresses.into_iter().map(|(address, chains)| AddressChains::new(address, chains.into_iter().collect())).collect(),
    }
}

struct WalletOverviewBuilder {
    wallet_id: i32,
    identifier: WalletId,
    source: WalletSource,
    addresses: BTreeSet<String>,
    chains: BTreeSet<Chain>,
    subscription_count: usize,
}
