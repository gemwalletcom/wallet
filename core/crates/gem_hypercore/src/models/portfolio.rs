use primitives::{DateTime, chart::ChartDateValue, portfolio::PerpetualPortfolioTimeframeData};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(from = "(i64, String)")]
pub struct HypercoreDataPoint {
    pub date: DateTime,
    pub value: f64,
}

impl From<(i64, String)> for HypercoreDataPoint {
    fn from((timestamp_ms, value): (i64, String)) -> Self {
        Self {
            date: DateTime::from_timestamp_millis(timestamp_ms),
            value: value.parse().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HypercorePortfolioTimeframeData {
    pub account_value_history: Vec<HypercoreDataPoint>,
    pub pnl_history: Vec<HypercoreDataPoint>,
    pub vlm: String,
}

impl From<HypercorePortfolioTimeframeData> for PerpetualPortfolioTimeframeData {
    fn from(data: HypercorePortfolioTimeframeData) -> Self {
        fn map_data_point(p: HypercoreDataPoint) -> ChartDateValue {
            ChartDateValue { date: p.date, value: p.value }
        }
        Self {
            account_value_history: data.account_value_history.into_iter().map(map_data_point).collect(),
            pnl_history: data.pnl_history.into_iter().map(map_data_point).collect(),
            volume: data.vlm.parse().unwrap_or(0.0),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct HypercorePortfolioResponse {
    pub timeframes: Vec<(String, HypercorePortfolioTimeframeData)>,
}
