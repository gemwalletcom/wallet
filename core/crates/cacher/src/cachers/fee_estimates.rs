use std::error::Error;

use async_trait::async_trait;
use primitives::{Chain, ChainFeeEstimates};
use strum::IntoEnumIterator;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait FeeEstimatesCacher: Send + Sync {
    async fn fresh_estimates(&self, chain: Chain) -> Result<Option<ChainFeeEstimates>, Box<dyn Error + Send + Sync>>;
    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl FeeEstimatesCacher for CacherClient {
    async fn fresh_estimates(&self, chain: Chain) -> Result<Option<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        let (cached, fresh) = futures::try_join!(
            self.get_cached_optional::<ChainFeeEstimates>(CacheKey::TransactionFeeEstimates(chain.as_ref())),
            self.get_cached_optional::<()>(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref())),
        )?;
        Ok(match (cached, fresh) {
            (Some(estimates), Some(())) => Some(estimates),
            (Some(_), None) | (None, _) => None,
        })
    }

    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set_cached(CacheKey::TransactionFeeEstimates(chain.as_ref()), estimates).await?;
        self.set_cached(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref()), &()).await
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        let keys = Chain::iter().map(|chain| CacheKey::TransactionFeeEstimates(chain.as_ref()).key()).collect();
        self.get_values::<Vec<ChainFeeEstimates>, ChainFeeEstimates>(keys).await
    }
}
