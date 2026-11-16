use chrono::{NaiveDateTime, NaiveTime, TimeDelta, Timelike};
use diesel::prelude::*;
use primitives::{Chain, FeeEstimateChartValue};

use crate::sql_types::ChainRow;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::fee_estimates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct FeeEstimateRow {
    pub chain: ChainRow,
    pub gas_price: f64,
    pub priority_fee: f64,
    pub created_at: NaiveDateTime,
}

impl FeeEstimateRow {
    pub fn from_primitive(chain: Chain, value: FeeEstimateChartValue) -> Self {
        let date = value.date.naive_utc();
        Self {
            chain: ChainRow::from(chain),
            gas_price: value.gas_price,
            priority_fee: value.priority_fee,
            created_at: date.date().and_time(NaiveTime::MIN) + TimeDelta::hours(date.hour().into()),
        }
    }

    pub fn as_primitive(&self) -> FeeEstimateChartValue {
        FeeEstimateChartValue {
            date: self.created_at.and_utc(),
            gas_price: self.gas_price,
            priority_fee: self.priority_fee,
        }
    }
}
