use date_time::deserialize_datetime_from_millis;
use primitives::{ChartCandleStick, ChartCandleUpdate, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candlestick {
    #[serde(deserialize_with = "deserialize_datetime_from_millis")]
    pub t: DateTime,
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
            date: c.t,
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
