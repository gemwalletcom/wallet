use num_bigint::BigInt;

use crate::decimal::{Decimal, Rounding, f64_parts};

const CENTS_PER_UNIT: i64 = 100;
const MANTISSA_LIMIT: u128 = 1 << 96;

pub fn percentage_suggestions(price: f64) -> Vec<i32> {
    let base = match price {
        p if p < 100.0 => 5,
        p if p < 10_000.0 => 3,
        _ => 2,
    };
    vec![base, base * 2, base * 3]
}

pub fn price_rounded_values(price: f64, by_percent: f64) -> Vec<f64> {
    let (Some(price), Some(by_percent)) = (decimal_from_f64(price), decimal_from_f64(by_percent)) else {
        return vec![];
    };
    if price < Decimal::new(1, 2) || by_percent <= Decimal::zero() {
        return vec![];
    }

    let cents_per_unit = Decimal::from(CENTS_PER_UNIT);
    let offset = by_percent / Decimal::from(100);
    let lower_target = &price * (Decimal::from(1) - &offset) * &cents_per_unit;
    let upper_target = &price * (Decimal::from(1) + &offset) * &cents_per_unit;
    let step = price_step_cents(&lower_target);
    let upper_rounding = if step > cents_per_unit { Rounding::HalfUp } else { Rounding::Up };

    let lower = (&lower_target / &step).with_scale_round(0, Rounding::Floor) * &step;
    let upper = (&upper_target / &step).with_scale_round(0, upper_rounding) * &step;

    [lower, upper]
        .into_iter()
        .filter(|cents| *cents > Decimal::zero())
        .filter_map(|cents| cents.to_i64())
        .map(|cents| cents as f64 / CENTS_PER_UNIT as f64)
        .collect()
}

fn decimal_from_f64(value: f64) -> Option<Decimal> {
    if !value.is_finite() {
        return None;
    }
    let (mantissa, mut exponent10) = f64_parts(value);
    if mantissa == 0 {
        return Some(Decimal::zero());
    }
    let mut mantissa = u128::from(mantissa);
    let mut exponent5 = -exponent10;
    while exponent5 > 0 {
        exponent5 -= 1;
        if mantissa & 1 == 0 || mantissa * 5 >= MANTISSA_LIMIT {
            exponent10 += 1;
            mantissa >>= 1;
        } else {
            mantissa *= 5;
        }
    }
    while exponent5 < 0 {
        if mantissa & (MANTISSA_LIMIT >> 1) == 0 {
            exponent10 -= 1;
            exponent5 += 1;
            mantissa <<= 1;
        } else if exponent10 * 2 > -exponent5 {
            return None;
        } else {
            exponent5 += 1;
            mantissa /= 5;
        }
    }
    while exponent10 > 0 {
        if mantissa * 10 >= MANTISSA_LIMIT {
            return None;
        }
        mantissa *= 10;
        exponent10 -= 1;
    }
    while exponent10 < -28 {
        let remainder = mantissa % 10;
        mantissa /= 10;
        exponent10 += 1;
        if mantissa == 0 {
            exponent10 = 0;
        } else if remainder >= 5 {
            mantissa += 1;
        }
    }
    while exponent10 < 0 && mantissa >= 1 << 52 {
        let remainder = mantissa % 10;
        mantissa /= 10;
        exponent10 += 1;
        if remainder >= 5 {
            mantissa += 1;
        }
    }
    while exponent10 < 0 && mantissa % 10 == 0 {
        mantissa /= 10;
        exponent10 += 1;
    }
    let digits = BigInt::from(mantissa);
    Some(Decimal::new(if value.is_sign_negative() { -digits } else { digits }, -i64::from(exponent10)))
}

fn price_step_cents(value_cents: &Decimal) -> Decimal {
    let step: i64 = match value_cents {
        v if *v < Decimal::from(100) => 1,
        v if *v < Decimal::from(10_000) => 100,
        v if *v < Decimal::from(50_000) => 1_000,
        v if *v < Decimal::from(1_000_000) => 5_000,
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

        assert_rounded(0.1 + 0.2, 10.0, [0.27, 0.33]);
        assert_rounded(14583399999.999998, 2.0, [14291732000.0, 14875068000.0]);
        assert_rounded(199999.99999999997, 0.5, [199000.0, 201000.0]);
        assert_eq!(price_rounded_values(1e27, 5.0), Vec::<f64>::new());
        assert_eq!(price_rounded_values(f64::NAN, 5.0), Vec::<f64>::new());
    }

    #[test]
    fn test_decimal_from_f64() {
        let converted = |value: f64| decimal_from_f64(value).map(|decimal| decimal.normalized().to_plain_string());
        assert_eq!(converted(0.1).as_deref(), Some("0.1"));
        assert_eq!(converted(0.2829).as_deref(), Some("0.2829"));
        assert_eq!(converted(0.1 + 0.2).as_deref(), Some("0.3"));
        assert_eq!(converted(123.456).as_deref(), Some("123.456"));
        assert_eq!(converted(1e-7).as_deref(), Some("0.0000001"));
        assert_eq!(converted(2.5e-10).as_deref(), Some("0.00000000025"));
        assert_eq!(converted(1e-30).as_deref(), Some("0"));
        assert_eq!(converted(5e-324).as_deref(), Some("0"));
        assert_eq!(converted(-2.5).as_deref(), Some("-2.5"));
        assert_eq!(converted(0.0).as_deref(), Some("0"));
        assert_eq!(converted(-0.0).as_deref(), Some("0"));
        assert_eq!(converted(14583399999.999998).as_deref(), Some("14583400000"));
        assert_eq!(converted(1e27).as_deref(), Some("1000000000000000013287555072"));
        assert_eq!(converted(7.9e28).as_deref(), Some("78999999999999996926548246528"));
        assert_eq!(converted(8e28), None);
        assert_eq!(converted(f64::MAX), None);
        assert_eq!(converted(f64::NAN), None);
        assert_eq!(converted(f64::INFINITY), None);
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
