use super::rules;
use crate::formatted_number::GemFormattedNumber;
use chrono::{DateTime, Utc};
use primitives::{ChartDateValue, Currency};

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemChartValueType {
    Price,
    PriceChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemChartDateStyle {
    Relative,
    DayTime,
    Day,
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
    pub bounds: GemChartBounds,
    pub date_style: GemChartDateStyle,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartSelection {
    pub header: GemChartHeader,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartBounds {
    pub lower_index: u32,
    pub upper_index: u32,
    pub y_min: f64,
    pub y_max: f64,
    pub low: GemFormattedNumber,
    pub high: GemFormattedNumber,
}

#[uniffi::export]
impl GemChartData {
    pub fn selection(&self, index: u32) -> Option<GemChartSelection> {
        let point = self.values.get(index as usize).filter(|point| (self.start..=self.end).contains(&point.date))?;
        Some(GemChartSelection {
            header: rules::header(self, point.value, None),
            date: point.date,
        })
    }

    pub fn index_at(&self, fraction: f64) -> Option<u32> {
        rules::nearest_index(self.values.iter().map(|point| point.date), self.start..=self.end, fraction)
    }
}
