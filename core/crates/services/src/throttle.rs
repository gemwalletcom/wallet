use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottledTask<'a> {
    FetchAssets { asset_id: &'a str },
    FetchNftAsset { asset_id: &'a str },
    FetchTransaction { chain: &'a str, hash: &'a str },
    FetchCoinAddresses { chain: &'a str, address: &'a str },
    FetchTokenAddresses { chain: &'a str, address: &'a str },
    FetchNftAssetsAddresses { chain: &'a str, address: &'a str },
    FetchAddressTransactions { chain: &'a str, address: &'a str },
    StakeRewardsAlert { chain: &'a str, address: &'a str },
    InactiveDeviceObservation { device_id: &'a str },
}

impl<'a> ThrottledTask<'a> {
    fn cache_key(self) -> CacheKey<'a> {
        match self {
            Self::FetchAssets { asset_id } => CacheKey::FetchAssets(asset_id),
            Self::FetchNftAsset { asset_id } => CacheKey::FetchNftAsset(asset_id),
            Self::FetchTransaction { chain, hash } => CacheKey::FetchTransaction(chain, hash),
            Self::FetchCoinAddresses { chain, address } => CacheKey::FetchCoinAddresses(chain, address),
            Self::FetchTokenAddresses { chain, address } => CacheKey::FetchTokenAddresses(chain, address),
            Self::FetchNftAssetsAddresses { chain, address } => CacheKey::FetchNftAssetsAddresses(chain, address),
            Self::FetchAddressTransactions { chain, address } => CacheKey::FetchAddressTransactions(chain, address),
            Self::StakeRewardsAlert { chain, address } => CacheKey::AlerterStakeRewards(chain, address),
            Self::InactiveDeviceObservation { device_id } => CacheKey::InactiveDeviceObserver(device_id),
        }
    }
}

#[async_trait]
pub trait Throttle: Send + Sync {
    async fn try_start(&self, task: ThrottledTask<'_>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn reset(&self, tasks: &[ThrottledTask<'_>]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl Throttle for CacherClient {
    async fn try_start(&self, task: ThrottledTask<'_>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.can_process_cached(task.cache_key()).await
    }

    async fn reset(&self, tasks: &[ThrottledTask<'_>]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let keys = tasks.iter().map(|task| task.cache_key().key()).collect::<Vec<_>>();
        self.delete_keys(&keys).await?;
        Ok(())
    }
}
