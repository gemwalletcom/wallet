use primitives::DateTime;
use serde::Deserialize;

use super::params::{ChainParam, ChartPeriodParam, CurrencyParam, lenient, timestamp};

#[derive(Deserialize)]
pub struct CurrencyQuery {
    #[serde(default)]
    pub currency: CurrencyParam,
}

#[derive(Deserialize)]
pub struct ChainQuery {
    pub chain: ChainParam,
}

#[derive(Deserialize)]
pub struct ChartPeriodQuery {
    #[serde(default, deserialize_with = "lenient")]
    pub period: Option<ChartPeriodParam>,
}

#[derive(Deserialize)]
pub struct FromTimestampQuery {
    #[serde(default, deserialize_with = "timestamp")]
    pub from_timestamp: Option<DateTime>,
}
