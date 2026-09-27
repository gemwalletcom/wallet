use chrono::{DateTime, Utc};
use model_derive::Model;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ChartCandleStick {
    pub date: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ChartCandleUpdate {
    pub coin: String,
    pub interval: String,
    pub candle: ChartCandleStick,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Model)]
#[model(swift = "Equatable, Sendable, Hashable")]
#[serde(rename_all = "camelCase")]
pub struct ChartDateValue {
    pub date: DateTime<Utc>,
    pub value: f64,
}
