use serde::Deserialize;

use super::params::{ChainParam, CurrencyParam, lenient};

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
pub struct FromTimestampQuery {
    #[serde(default, deserialize_with = "lenient")]
    pub from_timestamp: Option<u64>,
}
