use std::collections::HashSet;
use std::error::Error;

use cacher::{CacheKey, CacherClient};
use futures::future::join_all;
use primitives::{AddressStatus, Chain, ChainAddress, WalletConfiguration, WalletConfigurationResult, WalletId};
use settings_chain::ChainProviders;
use storage::{Database, WalletsRepository};

const ADDRESS_STATUS_CHAINS: [Chain; 6] = [Chain::Tron, Chain::Solana, Chain::Xrp, Chain::Stellar, Chain::Algorand, Chain::Aptos];

pub struct WalletConfigurationClient {
    database: Database,
    providers: ChainProviders,
    cacher: CacherClient,
}

impl WalletConfigurationClient {
    pub fn new(database: Database, providers: ChainProviders, cacher: CacherClient) -> Self {
        Self { database, providers, cacher }
    }

    pub async fn get_configuration(&self, device_id: i32, wallet_id: i32, wallet_identifier: WalletId) -> Result<WalletConfigurationResult, Box<dyn Error + Send + Sync>> {
        let externally_controlled_accounts = self.externally_controlled_accounts(device_id, wallet_id).await?;
        Ok(WalletConfigurationResult {
            wallet_id: wallet_identifier,
            configuration: WalletConfiguration {
                multi_signature_accounts: externally_controlled_accounts.clone(),
                externally_controlled_accounts,
            },
        })
    }

    async fn externally_controlled_accounts(&self, device_id: i32, wallet_id: i32) -> Result<Vec<ChainAddress>, Box<dyn Error + Send + Sync>> {
        Ok(join_all(
            self.subscribed_addresses(device_id, wallet_id)?
                .into_iter()
                .map(|address| async move { self.is_externally_controlled(&address).await.then_some(address) }),
        )
        .await
        .into_iter()
        .flatten()
        .collect())
    }

    async fn is_externally_controlled(&self, address: &ChainAddress) -> bool {
        self.get_statuses(address).await.is_some_and(|statuses| statuses.contains(&AddressStatus::ExternallyControlled))
    }

    fn subscribed_addresses(&self, device_id: i32, wallet_id: i32) -> Result<HashSet<ChainAddress>, Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .wallets()?
            .get_subscriptions_by_wallet_id(device_id, wallet_id)?
            .into_iter()
            .filter_map(|(subscription, address)| ADDRESS_STATUS_CHAINS.contains(&subscription.chain.0).then_some(ChainAddress::new(subscription.chain.0, address.address)))
            .collect())
    }

    async fn get_statuses(&self, address: &ChainAddress) -> Option<Vec<AddressStatus>> {
        if let Some(statuses) = self.cacher.get_cached_optional::<Vec<AddressStatus>>(cache_key(address)).await.ok().flatten().filter(|statuses| !statuses.is_empty()) {
            return Some(statuses);
        }

        let statuses = self.providers.get_address_status(address.chain, address.address.clone()).await.ok()?;
        if statuses.is_empty() {
            return None;
        }

        let _ = self.cacher.set_cached(cache_key(address), &statuses).await;

        Some(statuses)
    }
}

fn cache_key(address: &ChainAddress) -> CacheKey<'_> {
    CacheKey::AddressStatus(address.chain.as_ref(), &address.address)
}
