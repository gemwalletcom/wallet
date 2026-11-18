use async_trait::async_trait;
use primitives::{Chain, ChartPeriod, FeeEstimateChartValue};
use storage::{Database, DatabaseError, FeeEstimatesRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn add_fee_estimate(&self, chain: Chain, value: FeeEstimateChartValue) -> Result<usize, DatabaseError>;
    async fn get_fee_estimates(&self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn add_fee_estimate(&self, chain: Chain, value: FeeEstimateChartValue) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_fee_estimate(chain, value)).await
    }

    async fn get_fee_estimates(&self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        self.database.run(move |client| client.get_fee_estimates(chain, period)).await
    }
}
