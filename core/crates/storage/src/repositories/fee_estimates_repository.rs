use chrono::{NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::*;
use diesel::upsert::excluded;
use primitives::{Chain, ChartPeriod, FeeEstimateChartValue};

use crate::models::FeeEstimateRow;
use crate::sql_types::ChainRow;
use crate::{DatabaseClient, DatabaseError};

pub trait FeeEstimatesRepository {
    fn add_fee_estimates(&mut self, chain: Chain, values: Vec<FeeEstimateChartValue>) -> Result<usize, DatabaseError>;
    fn get_fee_estimates(&mut self, chain: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError>;
    fn delete_fee_estimates(&mut self, chain: Chain, before: NaiveDateTime) -> Result<usize, DatabaseError>;
}

impl FeeEstimatesRepository for DatabaseClient {
    fn add_fee_estimates(&mut self, chain_value: Chain, values: Vec<FeeEstimateChartValue>) -> Result<usize, DatabaseError> {
        use crate::schema::fee_estimates::dsl::*;
        use diesel::query_dsl::methods::FilterDsl;

        let rows = values.into_iter().map(|value| FeeEstimateRow::from_primitive(chain_value, value)).collect::<Vec<_>>();
        let insert = diesel::insert_into(fee_estimates)
            .values(&rows)
            .on_conflict((chain, priority, created_at))
            .do_update()
            .set((gas_price.eq(excluded(gas_price)), priority_fee.eq(excluded(priority_fee))));
        Ok(insert
            .filter(gas_price.is_distinct_from(excluded(gas_price)).or(priority_fee.is_distinct_from(excluded(priority_fee))))
            .execute(&mut self.connection)?)
    }

    fn get_fee_estimates(&mut self, chain_value: Chain, period: ChartPeriod) -> Result<Vec<FeeEstimateChartValue>, DatabaseError> {
        use crate::schema::fee_estimates::dsl::*;

        let since = Utc::now().naive_utc() - TimeDelta::minutes(period.minutes().into());
        Ok(fee_estimates
            .filter(chain.eq(ChainRow::from(chain_value)))
            .filter(created_at.ge(since))
            .order((created_at.asc(), priority.asc()))
            .select(FeeEstimateRow::as_select())
            .load(&mut self.connection)?
            .iter()
            .map(FeeEstimateRow::as_primitive)
            .collect())
    }

    fn delete_fee_estimates(&mut self, chain_value: Chain, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        use crate::schema::fee_estimates::dsl::*;
        Ok(diesel::delete(fee_estimates.filter(chain.eq(ChainRow::from(chain_value))).filter(created_at.lt(before))).execute(&mut self.connection)?)
    }
}

#[cfg(all(test, feature = "database_integration_tests"))]
mod database_integration_tests {
    use chrono::{DateTime, TimeDelta, Utc};
    use primitives::{Chain, ChartPeriod, FeeEstimateChartValue, FeePriority};

    use crate::{ChainsRepository, Database, DatabaseError, FeeEstimatesRepository};

    #[tokio::test]
    async fn test_add_fee_estimates() {
        let now = Utc::now();
        let value = FeeEstimateChartValue::mock_with_date(now);
        let changed = FeeEstimateChartValue { gas_price: 2.0, ..value.clone() };
        let fast = FeeEstimateChartValue {
            priority: FeePriority::Fast,
            ..value.clone()
        };
        let hour = DateTime::from_timestamp(now.timestamp() / 3600 * 3600, 0).unwrap();
        let expected = vec![FeeEstimateChartValue { date: hour, ..fast.clone() }, FeeEstimateChartValue { date: hour, ..changed.clone() }];

        let (writes, values, deleted) = Database::mock()
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_chains(vec![Chain::Ethereum])?;
                client.delete_fee_estimates(Chain::Ethereum, (now + TimeDelta::days(1)).naive_utc())?;
                let writes = vec![
                    client.add_fee_estimates(Chain::Ethereum, vec![value.clone(), fast.clone()])?,
                    client.add_fee_estimates(Chain::Ethereum, vec![value, fast.clone()])?,
                    client.add_fee_estimates(Chain::Ethereum, vec![changed, fast])?,
                ];
                let values = client.get_fee_estimates(Chain::Ethereum, ChartPeriod::Day)?;
                let deleted = client.delete_fee_estimates(Chain::Ethereum, (now + TimeDelta::hours(1)).naive_utc())?;
                Ok((writes, values, deleted))
            })
            .await
            .unwrap();

        assert_eq!(writes, vec![2, 0, 1]);
        assert_eq!(values, expected);
        assert_eq!(deleted, 2);
    }
}
