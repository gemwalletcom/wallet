use crate::{DateTime, FeeEstimateChartValue};

impl FeeEstimateChartValue {
    pub fn mock_with_date(date: DateTime) -> Self {
        Self { date, gas_price: 1.5, priority_fee: 0.1 }
    }
}
