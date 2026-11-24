use diesel::prelude::*;
use primitives::{Chain, FeeEstimateChartValue, HOUR};

use crate::sql_types::{ChainRow, DateTimeRow};

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = crate::schema::fee_estimates)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct FeeEstimateRow {
    pub chain: ChainRow,
    pub gas_price: f64,
    pub priority_fee: f64,
    pub created_at: DateTimeRow,
}

impl FeeEstimateRow {
    pub fn from_primitive(chain: Chain, value: FeeEstimateChartValue) -> Self {
        Self {
            chain: ChainRow::from(chain),
            gas_price: value.gas_price,
            priority_fee: value.priority_fee,
            created_at: value.date.truncated(HOUR).into(),
        }
    }

    pub fn as_primitive(&self) -> FeeEstimateChartValue {
        FeeEstimateChartValue {
            date: self.created_at.into(),
            gas_price: self.gas_price,
            priority_fee: self.priority_fee,
        }
    }
}
