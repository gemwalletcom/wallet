use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

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
        let key = vault_key(provider.as_ref(), kind);
        match kind {
            SwapVaultAddressKind::Deposit => self.add_to_expiring_set(key, addresses, SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64()).await,
            SwapVaultAddressKind::Send => self.add_to_set(key, addresses).await,
        }
    }

    async fn vault_addresses(&self, providers: &[SwapProvider], kind: SwapVaultAddressKind) -> Result<Vec<Vec<String>>, Box<dyn Error + Send + Sync>> {
        let keys = providers.iter().map(|provider| vault_key(provider.as_ref(), kind)).collect::<Vec<_>>();
        match kind {
            SwapVaultAddressKind::Deposit => self.expiring_set_members(&keys).await,
            SwapVaultAddressKind::Send => self.set_members(&keys).await,
        }
    }
}

fn vault_key(provider: &str, kind: SwapVaultAddressKind) -> CacheKey<'_> {
    match kind {
        SwapVaultAddressKind::Deposit => CacheKey::SwapDepositIds(provider),
        SwapVaultAddressKind::Send => CacheKey::SwapSendAddresses(provider),
    }
}

#[cfg(test)]
mod tests {
    use primitives::SECONDS_PER_DAY;

    use super::*;

    #[tokio::test]
    #[ignore = "requires REDIS_URL pointing at a disposable Redis instance"]
    async fn test_add_to_expiring_set() {
        let client = CacherClient::new(&std::env::var("REDIS_URL").unwrap()).await.unwrap();
        let tag = format!("expiring-test-{}-{}", std::process::id(), SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
        let key = CacheKey::SwapDepositIds(&tag);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64();

        client.add_to_expiring_set(key, &["expired".to_string()], now - (31 * SECONDS_PER_DAY) as f64).await.unwrap();
        client.add_to_expiring_set(key, &["recent".to_string()], now - SECONDS_PER_DAY as f64).await.unwrap();
        client.add_to_expiring_set(key, &["current".to_string()], now).await.unwrap();

        assert_eq!(client.expiring_set_members(&[key]).await.unwrap(), vec![vec!["recent".to_string(), "current".to_string()]]);
        client.delete(&[key]).await.unwrap();
    }
}
