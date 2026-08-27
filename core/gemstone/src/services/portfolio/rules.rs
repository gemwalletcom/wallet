use primitives::{ChartValuePercentage, PortfolioAssets};

use super::model::GemPortfolioValues;
use crate::services::chart::rules::converted_values;

pub fn converted_portfolio(portfolio: PortfolioAssets, rate: f64) -> GemPortfolioValues {
    GemPortfolioValues {
        values: converted_values(portfolio.values, rate),
        all_time_high: portfolio.all_time_high.map(|value| converted_percentage(value, rate)),
        all_time_low: portfolio.all_time_low.map(|value| converted_percentage(value, rate)),
    }
}

fn converted_percentage(value: ChartValuePercentage, rate: f64) -> ChartValuePercentage {
    ChartValuePercentage {
        date: value.date,
        value: (value.value as f64 * rate) as f32,
        percentage: value.percentage,
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use primitives::ChartValue;

    use super::*;

    #[test]
    fn test_converted_portfolio_applies_rate_to_values_and_extremes() {
        let now = Utc::now();
        let portfolio = PortfolioAssets {
            total_value: 10.0,
            values: vec![ChartValue { timestamp: 2, value: 2.0 }, ChartValue { timestamp: 1, value: 1.0 }],
            all_time_high: Some(ChartValuePercentage {
                date: now,
                value: 4.0,
                percentage: 10.0,
            }),
            all_time_low: None,
            allocation: vec![],
        };
        let converted = converted_portfolio(portfolio, 2.0);
        assert_eq!(converted.values.iter().map(|value| value.value).collect::<Vec<_>>(), vec![2.0, 4.0]);
        let high = converted.all_time_high.unwrap();
        assert_eq!(high.value, 8.0);
        assert_eq!(high.percentage, 10.0);
        assert!(converted.all_time_low.is_none());
    }
}
