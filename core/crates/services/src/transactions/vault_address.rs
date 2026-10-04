use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use primitives::SwapProvider;
use swapper::SwapperProvider;

type AddressMap = HashMap<String, SwapperProvider>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapVaultAddressKind {
    Deposit,
    Send,
}

#[async_trait]
pub trait SwapVaultAddressCacher: Send + Sync {
    async fn add_vault_addresses(&self, provider: SwapProvider, kind: SwapVaultAddressKind, addresses: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>>;
    async fn vault_addresses(&self, providers: &[SwapProvider], kind: SwapVaultAddressKind) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl SwapVaultAddressCacher for CacherClient {
    async fn add_vault_addresses(&self, provider: SwapProvider, kind: SwapVaultAddressKind, addresses: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.add_to_set_cached(vault_key(provider.as_ref(), kind), addresses).await
    }

    async fn vault_addresses(&self, providers: &[SwapProvider], kind: SwapVaultAddressKind) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>> {
        let keys = providers.iter().map(|provider| vault_key(provider.as_ref(), kind).key()).collect();
        self.get_set_members_grouped(keys).await
    }
}

fn vault_key(provider: &str, kind: SwapVaultAddressKind) -> CacheKey<'_> {
    match kind {
        SwapVaultAddressKind::Deposit => CacheKey::SwapDepositAddresses(provider),
        SwapVaultAddressKind::Send => CacheKey::SwapSendAddresses(provider),
    }
}

#[derive(Clone)]
pub struct SwapVaultAddressClient {
    cacher: Arc<dyn SwapVaultAddressCacher>,
}

impl SwapVaultAddressClient {
    pub fn new(cacher: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self { cacher }
    }

    pub async fn get_deposit_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(SwapVaultAddressKind::Deposit).await
    }

    pub async fn get_send_address_map(&self) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        self.get_address_map(SwapVaultAddressKind::Send).await
    }

    async fn get_address_map(&self, kind: SwapVaultAddressKind) -> Result<AddressMap, Box<dyn Error + Send + Sync>> {
        let providers = SwapProvider::cross_chain_providers();
        let results = self.cacher.vault_addresses(&providers, kind).await?;
        Ok(providers.into_iter().zip(results).flat_map(|(provider, members)| members.into_iter().map(move |address| (address, provider))).collect())
    }
}
