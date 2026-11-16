use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, ChainFeeEstimates};

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait FeeEstimatesCacher: Send + Sync {
    async fn get_estimates(&self, chain: Chain) -> Result<Option<ChainFeeEstimates>, Box<dyn Error + Send + Sync>>;
    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl FeeEstimatesCacher for CacherClient {
    async fn get_estimates(&self, chain: Chain) -> Result<Option<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        self.get_cached(CacheKey::TransactionFeeEstimates(chain.as_ref())).await
    }

    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::TransactionFeeEstimates(chain.as_ref()), estimates).await
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        let chains = Chain::all();
        let keys = chains.iter().map(|chain| CacheKey::TransactionFeeEstimates(chain.as_ref())).collect::<Vec<_>>();
        self.get_many_cached(&keys).await
    }
}
