use chrono::{DateTime, Utc};
use primitives::{ChartDateValue, ChartValue};

use crate::services::price::GemAssetPrice;

pub fn converted_values(prices: Vec<ChartValue>, rate: f64) -> Vec<ChartDateValue> {
    let mut values: Vec<ChartDateValue> = prices
        .into_iter()
        .filter_map(|price| {
            DateTime::from_timestamp(price.timestamp as i64, 0).map(|date| ChartDateValue {
                date,
                value: price.value as f64 * rate,
            })
        })
        .collect();
    values.sort_by_key(|value| value.date);
    values
}

pub fn current_value(values: &[ChartDateValue], latest: Option<GemAssetPrice>, now: DateTime<Utc>) -> Option<ChartDateValue> {
    let latest = latest?;
    let is_newer = values.last().is_none_or(|last| latest.updated_at > last.date);
    is_newer.then_some(ChartDateValue { date: now, value: latest.price })
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::AssetId;

    #[test]
    fn test_converted_values_apply_rate_and_sort() {
        let values = converted_values(vec![ChartValue { timestamp: 20, value: 2.0 }, ChartValue { timestamp: 10, value: 1.5 }], 2.0);
        assert_eq!(values.iter().map(|value| value.date.timestamp()).collect::<Vec<_>>(), vec![10, 20]);
        assert_eq!(values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![3.0, 4.0]);
    }

    #[test]
    fn test_current_value_is_only_a_price_newer_than_the_chart() {
        let point = |seconds: i64| ChartDateValue {
            date: DateTime::from_timestamp(seconds, 0).unwrap(),
            value: 1.0,
        };
        let price = |seconds: i64| GemAssetPrice {
            asset_id: AssetId::from_chain(primitives::Chain::Bitcoin),
            price: 9.0,
            price_change_percentage_24h: 0.0,
            updated_at: DateTime::from_timestamp(seconds, 0).unwrap(),
        };
        let now = DateTime::from_timestamp(500, 0).unwrap();

        let current = current_value(&[point(10), point(20)], Some(price(30)), now).expect("current");
        assert_eq!(current.value, 9.0);
        assert_eq!(current.date, now);

        assert_eq!(current_value(&[point(10), point(20)], Some(price(20)), now), None);
        assert_eq!(current_value(&[point(10), point(20)], None, now), None);
        assert!(current_value(&[], Some(price(20)), now).is_some());
    }
}
