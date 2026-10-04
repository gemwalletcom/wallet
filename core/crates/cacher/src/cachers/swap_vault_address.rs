use std::error::Error;

use async_trait::async_trait;
use primitives::SwapProvider;

use crate::{CacheKey, CacherClient};

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
