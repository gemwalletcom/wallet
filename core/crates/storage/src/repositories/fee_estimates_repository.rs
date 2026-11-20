use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{Chain, ChartPeriod, DateTime, FeeEstimateChartValue};
use std::time::Duration;

use crate::models::FeeEstimateRow;
use crate::sql_types::{ChainRow, DateTimeRow};
use crate::{DatabaseClient, DatabaseError};

pub trait FeeEstimatesRepository {
    fn add_fee_estimate(&mut self, chain: Chain, value: FeeEstimateChartValue) -> Result<usize, DatabaseError>;
    fn get_fee_estimates(&mut self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError>;
}

impl FeeEstimatesRepository for DatabaseClient {
    fn add_fee_estimate(&mut self, chain_value: Chain, value: FeeEstimateChartValue) -> Result<usize, DatabaseError> {
        use crate::schema::fee_estimates::dsl::*;
        use diesel::query_dsl::methods::FilterDsl;

        let insert = diesel::insert_into(fee_estimates)
            .values(FeeEstimateRow::from_primitive(chain_value, value))
            .on_conflict((chain, created_at))
            .do_update()
            .set((gas_price.eq(excluded(gas_price)), priority_fee.eq(excluded(priority_fee))));
        Ok(insert
            .filter(gas_price.is_distinct_from(excluded(gas_price)).or(priority_fee.is_distinct_from(excluded(priority_fee))))
            .execute(&mut self.connection)?)
    }

    fn get_fee_estimates(&mut self, chain_value: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        use crate::schema::fee_estimates::dsl::*;

        let since = DateTime::now() - Duration::from_mins(period.minutes());
        Ok(fee_estimates
            .filter(chain.eq(ChainRow::from(chain_value)))
            .filter(created_at.ge(DateTimeRow::from(since)))
            .order(created_at.asc())
            .select(FeeEstimateRow::as_select())
            .load(&mut self.connection)?
            .iter()
            .map(FeeEstimateRow::as_primitive)
            .collect())
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use diesel::RunQueryDsl;
    use primitives::{Chain, ChartPeriod, DateTime, FeeEstimateChartValue, HOUR};

    use crate::{ChainsRepository, Database, DatabaseError, FeeEstimatesRepository};

    #[tokio::test]
    async fn test_add_fee_estimate() {
        let now = DateTime::now();
        let value = FeeEstimateChartValue::mock_with_date(now);
        let changed = FeeEstimateChartValue { gas_price: 2.0, ..value.clone() };
        let expected = vec![FeeEstimateChartValue {
            date: now.truncated(HOUR),
            ..changed.clone()
        }];

        let (writes, values) = Database::mock()
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Ethereum])?;
                diesel::delete(crate::schema::fee_estimates::table).execute(&mut client.connection)?;
                let writes = vec![
                    client.add_fee_estimate(Chain::Ethereum, value.clone())?,
                    client.add_fee_estimate(Chain::Ethereum, value)?,
                    client.add_fee_estimate(Chain::Ethereum, changed)?,
                ];
                Ok((writes, client.get_fee_estimates(Chain::Ethereum, ChartPeriod::Day)?))
            })
            .await
            .unwrap();

        assert_eq!(writes, vec![1, 0, 1]);
        assert_eq!(values, expected);
    }
}
