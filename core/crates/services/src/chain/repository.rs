use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Chain, ChartPeriod, FeeEstimateChartValue};
use storage::{Database, DatabaseError, FeeEstimatesRepository};

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn add_fee_estimates(&self, chain: Chain, values: Vec<FeeEstimateChartValue>) -> Result<usize, DatabaseError>;
    async fn get_fee_estimates(&self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError>;
    async fn delete_fee_estimates(&self, chain: Chain, before: NaiveDateTime) -> Result<usize, DatabaseError>;
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
    async fn add_fee_estimates(&self, chain: Chain, values: Vec<FeeEstimateChartValue>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_fee_estimates(chain, values)).await
    }

    async fn get_fee_estimates(&self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        self.database.run(move |client| client.get_fee_estimates(chain, period)).await
    }

    async fn delete_fee_estimates(&self, chain: Chain, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.delete_fee_estimates(chain, before)).await
    }
}
