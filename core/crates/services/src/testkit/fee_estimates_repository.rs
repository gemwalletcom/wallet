use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{Chain, ChartPeriod, FeeEstimateChartValue};
use storage::DatabaseError;

use crate::chain::repository::Repository;

#[derive(Default)]
pub(crate) struct MemoryFeeEstimatesRepository {
    values: Mutex<Vec<(Chain, FeeEstimateChartValue)>>,
}

impl MemoryFeeEstimatesRepository {
    pub(crate) fn values(&self) -> Vec<(Chain, FeeEstimateChartValue)> {
        self.values.lock().unwrap().clone()
    }
}

#[async_trait]
impl Repository for MemoryFeeEstimatesRepository {
    async fn add_fee_estimate(&self, chain: Chain, value: FeeEstimateChartValue) -> Result<usize, DatabaseError> {
        self.values.lock().unwrap().push((chain, value));
        Ok(1)
    }

    async fn get_fee_estimates(&self, chain: Chain, _period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        Ok(self.values.lock().unwrap().iter().filter(|(value_chain, _)| *value_chain == chain).map(|(_, value)| value.clone()).collect())
    }
}
