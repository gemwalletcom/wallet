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
            self.get::<ChainFeeEstimates>(CacheKey::TransactionFeeEstimates(chain.as_ref())),
            self.get::<()>(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref())),
        )?;
        Ok(match (cached, fresh) {
            (Some(estimates), Some(())) => Some(estimates),
            (Some(_), None) | (None, _) => None,
        })
    }

    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::TransactionFeeEstimates(chain.as_ref()), estimates).await?;
        self.set(CacheKey::TransactionFeeEstimatesFresh(chain.as_ref()), &()).await
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        let chains = Chain::iter().collect::<Vec<_>>();
        let keys = chains.iter().map(|chain| CacheKey::TransactionFeeEstimates(chain.as_ref())).collect::<Vec<_>>();
        self.get_many(&keys).await
    }
}
