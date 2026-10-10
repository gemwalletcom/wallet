use std::{collections::HashMap, error::Error, sync::Mutex, time::Duration};

use async_trait::async_trait;
use cacher::{CacheFuture, FeeEstimatesCacher};
use primitives::{Chain, ChainFeeEstimates};

#[derive(Default)]
pub(crate) struct MemoryFeeEstimatesCacher {
    estimates: Mutex<HashMap<Chain, ChainFeeEstimates>>,
    fresh: Mutex<HashMap<Chain, bool>>,
    writes: Mutex<Vec<(Chain, Duration)>>,
}

impl MemoryFeeEstimatesCacher {
    pub(crate) fn with_estimates(self, chain: Chain, estimates: ChainFeeEstimates, fresh: bool) -> Self {
        self.estimates.lock().unwrap().insert(chain, estimates);
        self.fresh.lock().unwrap().insert(chain, fresh);
        self
    }

    pub(crate) fn writes(&self) -> Vec<(Chain, Duration)> {
        self.writes.lock().unwrap().clone()
    }
}

#[async_trait]
impl FeeEstimatesCacher for MemoryFeeEstimatesCacher {
    async fn get_or_fetch_estimates(&self, chain: Chain, ttl: Duration, fetch: CacheFuture<'_, ChainFeeEstimates>) -> Result<ChainFeeEstimates, Box<dyn Error + Send + Sync>> {
        let cached = self.estimates.lock().unwrap().get(&chain).cloned();
        if !ttl.is_zero()
            && self.fresh.lock().unwrap().get(&chain).copied().unwrap_or(false)
            && let Some(estimates) = cached
        {
            return Ok(estimates);
        }
        match fetch.await {
            Ok(estimates) => {
                self.estimates.lock().unwrap().insert(chain, estimates.clone());
                self.fresh.lock().unwrap().insert(chain, !ttl.is_zero());
                self.writes.lock().unwrap().push((chain, ttl));
                Ok(estimates)
            }
            Err(error) => cached.ok_or(error),
        }
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        Ok(self.estimates.lock().unwrap().values().cloned().collect())
    }
}
