use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::error::Error;
use std::sync::Arc;

use primitives::{AddressChains, Chain, WalletId, WalletSource, WalletSubscription, WalletSubscriptionChains};
use streamer::{ChainAddressPayload, StreamProducerQueue};

use super::admin_device::AdminWalletOverview;
use super::repository::Repository;
use crate::subscriptions::SubscriptionLookup;

#[derive(Clone)]
pub struct WalletsClient {
    repository: Arc<dyn Repository>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    subscription_lookup: Arc<SubscriptionLookup>,
}

impl WalletsClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, stream_producer: Arc<dyn StreamProducerQueue>, subscription_lookup: Arc<SubscriptionLookup>) -> Self {
        Self {
            repository,
            stream_producer,
            subscription_lookup,
        }
    }

    pub async fn get_subscriptions(&self, device_row_id: i32) -> Result<Vec<WalletSubscriptionChains>, Box<dyn Error + Send + Sync>> {
        let rows = self.repository.subscriptions(device_row_id).await?;

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
        let rows = self.repository.device_subscriptions(device_id.to_string()).await?;
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
        Ok(self.repository.wallet_overviews(device_row_id).await?)
    }

    pub async fn get_wallet_subscription(&self, device_id: &str, wallet_id: &str) -> Result<WalletSubscription, Box<dyn Error + Send + Sync>> {
        let (wallet, rows) = self.repository.device_wallet_subscriptions(device_id.to_string(), wallet_id.to_string()).await?;
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
        let addresses = wallet_subscriptions
            .iter()
            .flat_map(WalletSubscription::chain_addresses)
            .map(|address| (address.chain, address.address))
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        self.subscription_lookup.cache_subscribed(&addresses).await?;
        let count = self.repository.add_subscriptions(device_row_id, wallet_subscriptions).await?;

        if !payload.is_empty() {
            self.stream_producer.publish_new_addresses(payload).await?;
        }

        Ok(count)
    }

    pub async fn delete_subscriptions(&self, device_row_id: i32, subscriptions: Vec<WalletSubscriptionChains>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if subscriptions.is_empty() {
            return Ok(0);
        }

        Ok(self.repository.delete_subscriptions(device_row_id, subscriptions).await?)
    }
}

fn wallet_subscription(wallet_id: WalletId, source: WalletSource, addresses: BTreeMap<String, BTreeSet<Chain>>) -> WalletSubscription {
    WalletSubscription {
        wallet_id,
        source: Some(source),
        subscriptions: addresses.into_iter().map(|(address, chains)| AddressChains::new(address, chains.into_iter().collect())).collect(),
    }
}
