use chrono::{DateTime, Utc};
use primitives::chart::{ChartCandleStick, ChartCandleUpdate};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    pub t: u64,
    pub s: String,
    pub i: String,
    pub o: String,
    pub h: String,
    pub l: String,
    pub c: String,
    pub v: String,
}

impl From<&Candlestick> for ChartCandleStick {
    fn from(c: &Candlestick) -> Self {
        ChartCandleStick {
            date: DateTime::from_timestamp(c.t as i64 / 1000, 0).unwrap_or_else(Utc::now),
            open: c.o.parse().unwrap_or(0.0),
            high: c.h.parse().unwrap_or(0.0),
            low: c.l.parse().unwrap_or(0.0),
            close: c.c.parse().unwrap_or(0.0),
            volume: c.v.parse().unwrap_or(0.0),
        }
    }
}

impl From<Candlestick> for ChartCandleUpdate {
    fn from(c: Candlestick) -> Self {
        ChartCandleUpdate {
            coin: c.s.clone(),
            interval: c.i.clone(),
            candle: ChartCandleStick::from(&c),
        }
    }
}
