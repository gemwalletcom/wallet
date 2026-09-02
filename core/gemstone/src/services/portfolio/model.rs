use primitives::{ChartDateValue, ChartValuePercentage};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPortfolioValues {
    pub values: Vec<ChartDateValue>,
    pub all_time_high: Option<ChartValuePercentage>,
    pub all_time_low: Option<ChartValuePercentage>,
}
