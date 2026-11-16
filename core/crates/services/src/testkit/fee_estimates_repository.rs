use std::sync::Mutex;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Chain, ChartPeriod, FeeEstimateChartValue};
use storage::DatabaseError;

use crate::chain::repository::Repository;

#[derive(Default)]
pub(crate) struct MemoryFeeEstimatesRepository {
    values: Mutex<Vec<(Chain, FeeEstimateChartValue)>>,
}

#[async_trait]
impl Repository for MemoryFeeEstimatesRepository {
    async fn add_fee_estimates(&self, chain: Chain, values: Vec<FeeEstimateChartValue>) -> Result<usize, DatabaseError> {
        let count = values.len();
        self.values.lock().unwrap().extend(values.into_iter().map(|value| (chain, value)));
        Ok(count)
    }

    async fn get_fee_estimates(&self, chain: Chain, _period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        Ok(self.values.lock().unwrap().iter().filter(|(value_chain, _)| *value_chain == chain).map(|(_, value)| value.clone()).collect())
    }

    async fn delete_fee_estimates(&self, chain: Chain, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        let mut values = self.values.lock().unwrap();
        let count = values.len();
        values.retain(|(value_chain, value)| *value_chain != chain || value.date.naive_utc() >= before);
        Ok(count - values.len())
    }
}
