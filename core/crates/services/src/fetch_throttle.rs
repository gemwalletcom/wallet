use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThrottledFetch<'a> {
    Assets { asset_id: &'a str },
    NftAsset { asset_id: &'a str },
    Transaction { chain: &'a str, hash: &'a str },
    CoinAddresses { chain: &'a str, address: &'a str },
    TokenAddresses { chain: &'a str, address: &'a str },
    NftAssetsAddresses { chain: &'a str, address: &'a str },
    AddressTransactions { chain: &'a str, address: &'a str },
}

impl<'a> ThrottledFetch<'a> {
    fn cache_key(self) -> CacheKey<'a> {
        match self {
            Self::Assets { asset_id } => CacheKey::FetchAssets(asset_id),
            Self::NftAsset { asset_id } => CacheKey::FetchNftAsset(asset_id),
            Self::Transaction { chain, hash } => CacheKey::FetchTransaction(chain, hash),
            Self::CoinAddresses { chain, address } => CacheKey::FetchCoinAddresses(chain, address),
            Self::TokenAddresses { chain, address } => CacheKey::FetchTokenAddresses(chain, address),
            Self::NftAssetsAddresses { chain, address } => CacheKey::FetchNftAssetsAddresses(chain, address),
            Self::AddressTransactions { chain, address } => CacheKey::FetchAddressTransactions(chain, address),
        }
    }
}

#[async_trait]
pub trait FetchThrottle: Send + Sync {
    async fn try_start(&self, fetch: ThrottledFetch<'_>) -> Result<bool, Box<dyn Error + Send + Sync>>;
    async fn reset(&self, fetches: &[ThrottledFetch<'_>]) -> Result<(), Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl FetchThrottle for CacherClient {
    async fn try_start(&self, fetch: ThrottledFetch<'_>) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.can_process_cached(fetch.cache_key()).await
    }

    async fn reset(&self, fetches: &[ThrottledFetch<'_>]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let keys = fetches.iter().map(|fetch| fetch.cache_key().key()).collect::<Vec<_>>();
        self.delete_keys(&keys).await?;
        Ok(())
    }
}
