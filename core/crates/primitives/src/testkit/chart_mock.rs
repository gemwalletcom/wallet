use chrono::DateTime;

use crate::{ChartCandleStick, ChartDateValue};

impl ChartDateValue {
    pub fn mock(seconds: i64, value: f64) -> Self {
        Self {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            value,
        }
    }
}

impl ChartCandleStick {
    pub fn mock(seconds: i64, close: f64) -> Self {
        Self {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            open: close - 1.0,
            high: close + 1.0,
            low: close - 2.0,
            close,
            volume: 1000.0,
        }
    }

    pub fn mock_range(low: f64, high: f64) -> Self {
        Self {
            date: DateTime::UNIX_EPOCH,
            open: low,
            high,
            low,
            close: high,
            volume: 0.0,
        }
    }
}
