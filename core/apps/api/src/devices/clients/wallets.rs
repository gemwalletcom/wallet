use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::error::Error;

use primitives::{AddressChains, Chain, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};
use storage::models::NewWalletRow;
use storage::sql_types::WalletType;
use storage::{Database, DevicesRepository, WalletsRepository};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue};

#[derive(Clone)]
pub struct WalletsClient {
    database: Database,
    stream_producer: StreamProducer,
}

impl WalletsClient {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub fn get_subscriptions(&self, device_row_id: i32) -> Result<Vec<WalletSubscriptionChains>, Box<dyn Error + Send + Sync>> {
        let rows = self.database.wallets()?.get_subscriptions(device_row_id)?;

        Ok(rows
            .into_iter()
            .fold(
                BTreeMap::<String, (WalletId, Vec<Chain>)>::new(),
                |mut acc, (wallet_row, subscription_row, _address_row)| {
                    let wallet_id = wallet_row.wallet_id.0.clone();
                    acc.entry(wallet_id.id()).or_insert((wallet_id, Vec::new())).1.push(subscription_row.chain.0);
                    acc
                },
            )
            .into_values()
            .map(|(wallet_id, mut chains)| {
                chains.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));
                WalletSubscriptionChains { wallet_id, chains }
            })
            .collect())
    }

    pub fn get_wallet_subscriptions(&self, device_id: &str) -> Result<Vec<WalletSubscription>, Box<dyn Error + Send + Sync>> {
        let device_row_id = self.database.devices()?.get_device_row_id(device_id)?;
        let rows = self.database.wallets()?.get_subscriptions(device_row_id)?;
        let mut subscriptions = BTreeMap::<String, (WalletId, WalletSource, BTreeMap<String, BTreeSet<Chain>>)>::new();

        for (wallet, subscription, address) in rows {
            subscriptions
                .entry(wallet.wallet_id.0.id())
                .or_insert_with(|| (wallet.wallet_id.0, wallet.source.0, BTreeMap::new()))
                .2
                .entry(address.address)
                .or_default()
                .insert(subscription.chain.0);
        }

        Ok(subscriptions
            .into_values()
            .map(|(wallet_id, source, addresses)| WalletSubscription {
                wallet_id,
                source: Some(source),
                subscriptions: addresses
                    .into_iter()
                    .map(|(address, chains)| AddressChains::new(address, chains.into_iter().collect()))
                    .collect(),
            })
            .collect())
    }

    pub async fn add_subscriptions(&self, device_row_id: i32, wallet_subscriptions: Vec<WalletSubscription>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if wallet_subscriptions.is_empty() {
            return Ok(0);
        }

        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = wallet_subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let mut wallet_ids: HashMap<String, i32> = store.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

        let new_wallets: Vec<NewWalletRow> = wallet_subscriptions
            .iter()
            .filter(|x| !wallet_ids.contains_key(&x.wallet_id.id()))
            .map(|x| NewWalletRow {
                identifier: x.wallet_id.id(),
                wallet_type: WalletType::from(x.wallet_id.wallet_type()),
                source: storage::sql_types::WalletSource::from(x.source.clone().unwrap_or(WalletSource::Import)),
            })
            .collect();

        if !new_wallets.is_empty() {
            let new_identifiers: Vec<String> = new_wallets.iter().map(|x| x.identifier.clone()).collect();
            store.create_wallets(new_wallets)?;
            wallet_ids.extend(store.get_wallets(new_identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)));
        }

        let subscriptions: Vec<(i32, Chain, String)> = wallet_subscriptions
            .iter()
            .filter_map(|ws| {
                wallet_ids
                    .get(&ws.wallet_id.id())
                    .map(|&wallet_id| ws.chain_addresses().into_iter().map(move |ca| (wallet_id, ca.chain, ca.address)))
            })
            .flatten()
            .collect();

        let count = store.add_subscriptions(device_row_id, subscriptions)?;

        let payload: Vec<ChainAddressPayload> = wallet_subscriptions
            .into_iter()
            .filter(|x| x.source == Some(WalletSource::Import))
            .flat_map(|x| x.chain_addresses())
            .map(ChainAddressPayload::from)
            .collect();

        if !payload.is_empty() {
            self.stream_producer.publish_new_addresses(payload).await?;
        }

        Ok(count)
    }

    pub async fn delete_subscriptions(&self, device_row_id: i32, subscriptions: Vec<WalletSubscriptionChains>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        let mut store = self.database.wallets()?;

        let identifiers: Vec<String> = subscriptions.iter().map(|x| x.wallet_id.id()).collect();
        let wallet_ids: HashMap<String, i32> = store.get_wallets(identifiers)?.into_iter().map(|x| (x.wallet_id.id(), x.id)).collect();

        let mut count = 0;
        for ws in subscriptions {
            if let Some(&wallet_id) = wallet_ids.get(&ws.wallet_id.id()) {
                count += store.delete_wallet_chains(device_row_id, wallet_id, ws.chains)?;
            }
        }

        Ok(count)
    }
}
