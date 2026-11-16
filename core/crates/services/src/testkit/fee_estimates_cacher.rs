use std::{collections::HashMap, error::Error, sync::Mutex};

use async_trait::async_trait;
use cacher::FeeEstimatesCacher;
use primitives::{Chain, ChainFeeEstimates};

#[derive(Default)]
pub(crate) struct MemoryFeeEstimatesCacher {
    estimates: Mutex<HashMap<Chain, ChainFeeEstimates>>,
    writes: Mutex<Vec<Chain>>,
}

impl MemoryFeeEstimatesCacher {
    pub(crate) fn with_estimates(self, chain: Chain, estimates: ChainFeeEstimates) -> Self {
        self.estimates.lock().unwrap().insert(chain, estimates);
        self
    }

    pub(crate) fn writes(&self) -> Vec<Chain> {
        self.writes.lock().unwrap().clone()
    }
}

#[async_trait]
impl FeeEstimatesCacher for MemoryFeeEstimatesCacher {
    async fn get_estimates(&self, chain: Chain) -> Result<Option<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        Ok(self.estimates.lock().unwrap().get(&chain).cloned())
    }

    async fn set_estimates(&self, chain: Chain, estimates: &ChainFeeEstimates) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.estimates.lock().unwrap().insert(chain, estimates.clone());
        self.writes.lock().unwrap().push(chain);
        Ok(())
    }

    async fn all_estimates(&self) -> Result<Vec<ChainFeeEstimates>, Box<dyn Error + Send + Sync>> {
        Ok(self.estimates.lock().unwrap().values().cloned().collect())
    }
}
