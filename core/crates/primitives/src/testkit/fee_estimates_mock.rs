use chrono::{DateTime, Utc};

use crate::{FeeEstimateChartValue, FeePriority};

impl FeeEstimateChartValue {
    pub fn mock_with_date(date: DateTime<Utc>) -> Self {
        Self {
            date,
            priority: FeePriority::Normal,
            gas_price: 1.5,
            priority_fee: 0.1,
        }
    }
}
