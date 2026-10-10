use std::error::Error;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{Chain, ChainFeeEstimates};
use strum::IntoEnumIterator;

use crate::cache::Cached;
use crate::{CacheFuture, CacheKey, CacherClient};

#[async_trait]
pub trait FeeEstimatesCacher: Send + Sync {
    async fn get_or_fetch_estimates(&self, chain: Chain, duration: Duration, fetch: CacheFuture<'_, ChainFeeEstimates>) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>>;
    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl FeeEstimatesCacher for CacherClient {
    async fn get_or_fetch_estimates(&self, chain: Chain, duration: Duration, fetch: CacheFuture<'_, ChainFeeEstimates>) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
        self.get_or_fetch_with_fallback(CacheKey::TransactionFeeEstimates(chain.as_ref()), duration, fetch).await
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        let chains = Chain::iter().collect::<Vec<_>>();
        let keys = chains.iter().map(|chain| CacheKey::TransactionFeeEstimates(chain.as_ref())).collect::<Vec<_>>();
        Ok(self.get_many::<Cached<ChainFeeEstimates>>(&keys).await?.into_iter().map(|cached| cached.value).collect())
    }
}
