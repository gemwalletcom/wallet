use chrono::{DateTime, TimeDelta, Utc};
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

pub trait ChartCandleStickVecExt {
    fn interval(&self) -> Option<TimeDelta>;
    fn merged(&self, max_count: usize) -> Vec<ChartCandleStick>;
}

impl ChartCandleStickVecExt for [ChartCandleStick] {
    fn interval(&self) -> Option<TimeDelta> {
        self.windows(2).map(|pair| pair[1].date - pair[0].date).filter(|gap| *gap > TimeDelta::zero()).min()
    }

    fn merged(&self, max_count: usize) -> Vec<ChartCandleStick> {
        let Some(interval) = self.interval() else {
            return self.to_vec();
        };
        let bucket = (interval * self.len().div_ceil(max_count.max(1)) as i32).num_milliseconds();
        self.iter().fold(Vec::new(), |mut merged: Vec<ChartCandleStick>, candle| {
            let start = DateTime::from_timestamp_millis(candle.date.timestamp_millis().div_euclid(bucket) * bucket).unwrap_or(candle.date);
            match merged.last_mut().filter(|last| last.date == start) {
                Some(last) => {
                    last.high = last.high.max(candle.high);
                    last.low = last.low.min(candle.low);
                    last.close = candle.close;
                    last.volume += candle.volume;
                }
                None => merged.push(ChartCandleStick { date: start, ..candle.clone() }),
            }
            merged
        })
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merged() {
        let month = ChartCandleStick::mock_series(DateTime::from_timestamp(4 * 3_600, 0).unwrap(), TimeDelta::hours(4), 180);
        let merged = month.merged(60);

        assert_eq!(merged.len(), 61, "three four-hour candles per twelve hours, with a partial bucket at each end");
        assert_eq!(merged.interval(), Some(TimeDelta::hours(12)));
        assert_eq!(merged.windows(2).map(|pair| pair[1].date - pair[0].date).max(), Some(TimeDelta::hours(12)), "every merged candle is equally wide");
        assert_eq!(
            merged[1],
            ChartCandleStick {
                date: DateTime::from_timestamp(12 * 3_600, 0).unwrap(),
                open: 101.0,
                high: 105.0,
                low: 100.0,
                close: 104.0,
                volume: 3000.0,
            }
        );

        let day = ChartCandleStick::mock_series(DateTime::UNIX_EPOCH, TimeDelta::minutes(30), 48);
        assert_eq!(day.merged(60), day, "a period under the limit keeps every candle");
        assert_eq!([ChartCandleStick::mock(0, 1.0)].merged(60), vec![ChartCandleStick::mock(0, 1.0)]);
    }
}
