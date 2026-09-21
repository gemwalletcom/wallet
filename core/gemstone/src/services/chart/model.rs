use super::rules;
use crate::formatted_number::GemFormattedNumber;
use primitives::{ChartDateValue, Currency};

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemChartValueType {
    Price,
    PriceChange,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartHeader {
    pub value: GemFormattedNumber,
    pub secondary_value: Option<GemFormattedNumber>,
    pub change: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartData {
    pub value_type: GemChartValueType,
    pub base: f64,
    pub shows_secondary_value: bool,
    pub currency: Currency,
    pub values: Vec<ChartDateValue>,
    pub header: Option<GemChartHeader>,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemChartBounds {
    pub lower_index: u32,
    pub upper_index: u32,
    pub y_min: f64,
    pub y_max: f64,
}

#[uniffi::export]
impl GemChartData {
    pub fn header_at(&self, value: f64) -> GemChartHeader {
        rules::header(self, value, None)
    }

    pub fn bounds(&self) -> GemChartBounds {
        rules::chart_bounds(&self.values)
    }
}
