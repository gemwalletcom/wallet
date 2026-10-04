use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chain_providers::ChainProviders;
use futures::future::join_all;
use primitives::{AddressStatus, Chain, ChainAddress, WalletConfiguration, WalletConfigurationResult, WalletId, WalletType};
use storage::{Database, WalletsRepository};

const ADDRESS_STATUS_CHAINS: [Chain; 7] = [Chain::Tron, Chain::Solana, Chain::Xrp, Chain::Stellar, Chain::Algorand, Chain::Aptos, Chain::Near];

#[async_trait]
pub trait AddressStatusCacher: Send + Sync {
    async fn address_statuses(&self, address: &ChainAddress) -> Result<Option<Vec<AddressStatus>>, Box<dyn Error + Send + Sync>>;
    async fn set_address_statuses(&self, address: &ChainAddress, statuses: &[AddressStatus]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl AddressStatusCacher for CacherClient {
    async fn address_statuses(&self, address: &ChainAddress) -> Result<Option<Vec<AddressStatus>>, Box<dyn Error + Send + Sync>> {
        self.get_cached_optional(cache_key(address)).await
    }

    async fn set_address_statuses(&self, address: &ChainAddress, statuses: &[AddressStatus]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(cache_key(address), &statuses).await
    }
}

pub struct WalletConfigurationClient {
    database: Database,
    providers: ChainProviders,
    statuses: Arc<dyn AddressStatusCacher>,
}

impl WalletConfigurationClient {
    pub fn new(database: Database, providers: ChainProviders, statuses: Arc<dyn AddressStatusCacher>) -> Self {
        Self { database, providers, statuses }
    }

    pub async fn get_configuration(&self, device_id: i32, wallet_id: i32, wallet_identifier: WalletId, wallet_type: WalletType) -> Result<WalletConfigurationResult, Box<dyn Error + Send + Sync>> {
        let externally_controlled_accounts = match wallet_type.can_sign() {
            true => self.externally_controlled_accounts(device_id, wallet_id).await?,
            false => vec![],
        };
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
            self.subscribed_addresses(device_id, wallet_id)
                .await?
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

    async fn subscribed_addresses(&self, device_id: i32, wallet_id: i32) -> Result<HashSet<ChainAddress>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_id, wallet_id)).await?;
        Ok(subscriptions.into_iter().filter(|subscription| ADDRESS_STATUS_CHAINS.contains(&subscription.chain)).collect())
    }

    async fn get_statuses(&self, address: &ChainAddress) -> Option<Vec<AddressStatus>> {
        if let Some(statuses) = self.statuses.address_statuses(address).await.ok().flatten().filter(|statuses| !statuses.is_empty()) {
            return Some(statuses);
        }

        let statuses = self.providers.get_address_status(address.chain, address.address.clone()).await.ok()?;
        if statuses.is_empty() {
            return None;
        }

        let _ = self.statuses.set_address_statuses(address, &statuses).await;

        Some(statuses)
    }
}

fn cache_key(address: &ChainAddress) -> CacheKey<'_> {
    CacheKey::AddressStatus(address.chain.as_ref(), &address.address)
}
