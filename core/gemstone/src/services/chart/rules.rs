use chrono::DateTime;
use primitives::{ChartDateValue, ChartValue};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converted_values_apply_rate_and_sort() {
        let values = converted_values(vec![ChartValue { timestamp: 20, value: 2.0 }, ChartValue { timestamp: 10, value: 1.5 }], 2.0);
        assert_eq!(values.iter().map(|value| value.date.timestamp()).collect::<Vec<_>>(), vec![10, 20]);
        assert_eq!(values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![3.0, 4.0]);
    }
}
