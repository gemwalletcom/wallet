use std::error::Error;

use async_trait::async_trait;
use primitives::Chain;

use crate::{CacheKey, CacherClient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerpetualAddressTier {
    Tracked,
    Active,
    Priority,
}

#[async_trait]
pub trait PerpetualAddressCacher: Send + Sync {
    async fn addresses(&self, chain: Chain, tier: PerpetualAddressTier) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;
    async fn set_addresses(&self, chain: Chain, tier: PerpetualAddressTier, addresses: &[String]) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn checkpoint(&self, chain: Chain, address: &str) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>;
    async fn set_checkpoint(&self, chain: Chain, address: &str, timestamp: u64) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl PerpetualAddressCacher for CacherClient {
    async fn addresses(&self, chain: Chain, tier: PerpetualAddressTier) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        Ok(self.get_cached_optional::<Vec<String>>(tier_key(&chain, tier)).await?.unwrap_or_default())
    }

    async fn set_addresses(&self, chain: Chain, tier: PerpetualAddressTier, addresses: &[String]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(tier_key(&chain, tier), &addresses).await
    }

    async fn checkpoint(&self, chain: Chain, address: &str) -> Result<Option<u64>, Box<dyn Error + Send + Sync>> {
        self.get_value_optional(&CacheKey::PerpetualObserverCheckpoint(chain.as_ref(), address).key()).await
    }

    async fn set_checkpoint(&self, chain: Chain, address: &str, timestamp: u64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let checkpoint = CacheKey::PerpetualObserverCheckpoint(chain.as_ref(), address);
        self.set_value_with_ttl(&checkpoint.key(), timestamp.to_string(), checkpoint.ttl()).await
    }
}

fn tier_key(chain: &Chain, tier: PerpetualAddressTier) -> CacheKey<'_> {
    match tier {
        PerpetualAddressTier::Tracked => CacheKey::PerpetualTrackedAddresses(chain.as_ref()),
        PerpetualAddressTier::Active => CacheKey::PerpetualActiveAddresses(chain.as_ref()),
        PerpetualAddressTier::Priority => CacheKey::PerpetualPriorityAddresses(chain.as_ref()),
    }
}
