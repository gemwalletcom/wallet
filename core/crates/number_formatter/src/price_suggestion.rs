use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, RoundingStrategy};

const CENTS_PER_UNIT: i64 = 100;

pub fn percentage_suggestions(price: f64) -> Vec<i32> {
    let base = match price {
        p if p < 100.0 => 5,
        p if p < 10_000.0 => 3,
        _ => 2,
    };
    vec![base, base * 2, base * 3]
}

pub fn price_rounded_values(price: f64, by_percent: f64) -> Vec<f64> {
    let (Some(price), Some(by_percent)) = (Decimal::from_f64(price), Decimal::from_f64(by_percent)) else {
        return vec![];
    };
    if price < Decimal::new(1, 2) || by_percent <= Decimal::ZERO {
        return vec![];
    }

    let cents_per_unit = Decimal::from(CENTS_PER_UNIT);
    let offset = by_percent / Decimal::ONE_HUNDRED;
    let lower_target = price * (Decimal::ONE - offset) * cents_per_unit;
    let upper_target = price * (Decimal::ONE + offset) * cents_per_unit;
    let step = price_step_cents(lower_target);
    let upper_rounding = if step > cents_per_unit { RoundingStrategy::MidpointAwayFromZero } else { RoundingStrategy::AwayFromZero };

    let lower = (lower_target / step).floor() * step;
    let upper = (upper_target / step).round_dp_with_strategy(0, upper_rounding) * step;

    [lower, upper]
        .into_iter()
        .filter(|cents| *cents > Decimal::ZERO)
        .filter_map(|cents| cents.to_i64())
        .map(|cents| cents as f64 / CENTS_PER_UNIT as f64)
        .collect()
}

fn price_step_cents(value_cents: Decimal) -> Decimal {
    let step: i64 = match value_cents {
        v if v < Decimal::from(100) => 1,
        v if v < Decimal::from(10_000) => 100,
        v if v < Decimal::from(50_000) => 1_000,
        v if v < Decimal::from(1_000_000) => 5_000,
        _ => 100_000,
    };
    Decimal::from(step)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_rounded(price: f64, by_percent: f64, expected: [f64; 2]) {
        let result = price_rounded_values(price, by_percent);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_percentage_suggestions() {
        assert_eq!(percentage_suggestions(50.0), vec![5, 10, 15]);
        assert_eq!(percentage_suggestions(500.0), vec![3, 6, 9]);
        assert_eq!(percentage_suggestions(10_000.0), vec![2, 4, 6]);
    }

    #[test]
    fn test_price_rounded_values() {
        assert_rounded(0.2829, 5.0, [0.26, 0.30]);
        assert_rounded(767.55, 5.0, [700.0, 800.0]);
        assert_rounded(95_432.0, 5.0, [90_000.0, 100_000.0]);

        assert_rounded(0.66, 5.0, [0.62, 0.70]);
        assert_rounded(0.78, 5.0, [0.74, 0.82]);
        assert_eq!(price_rounded_values(0.01, 5.0), vec![0.02]);
        assert_rounded(0.2, 5.0, [0.19, 0.21]);
        assert_rounded(1.0, 5.0, [0.95, 1.05]);
        assert_rounded(500.0, 5.0, [470.0, 530.0]);

        assert!(price_rounded_values(0.0, 5.0).is_empty());
        assert!(price_rounded_values(100.0, -5.0).is_empty());
    }

    #[test]
    fn test_price_rounded_values_print_as_typed() {
        for cents in 1..=10_000 {
            let price = cents as f64 / 100.0;
            for value in price_rounded_values(price, 5.0) {
                let text = value.to_string();
                assert!(text.split('.').nth(1).is_none_or(|fraction| fraction.len() <= 2), "{price} suggested {text}");
            }
        }
    }
}
