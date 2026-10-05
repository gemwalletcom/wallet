use bigdecimal::{BigDecimal, RoundingMode, ToPrimitive};
use num_bigint::BigUint;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
pub enum NumberFormatterError {
    InvalidNumber(String),
    ConversionError(String),
}

impl std::fmt::Display for NumberFormatterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidNumber(msg) => write!(f, "Invalid number: {}", msg),
            Self::ConversionError(msg) => write!(f, "Conversion error: {}", msg),
        }
    }
}

impl std::error::Error for NumberFormatterError {}

impl From<NumberFormatterError> for String {
    fn from(error: NumberFormatterError) -> Self {
        error.to_string()
    }
}

pub struct BigNumberFormatter {}

impl BigNumberFormatter {
    pub(crate) fn scaled_value(value: impl Display, decimals: u32) -> Result<BigDecimal, NumberFormatterError> {
        let (digits, scale) = BigDecimal::from_str(&value.to_string()).map_err(|error| NumberFormatterError::InvalidNumber(error.to_string()))?.into_bigint_and_exponent();
        Ok(BigDecimal::new(digits, scale + i64::from(decimals)))
    }

    pub fn big_decimal_value(value: impl Display, decimals: u32) -> Result<BigDecimal, NumberFormatterError> {
        let decimal = Self::scaled_value(value, decimals)?.normalized();
        Ok(if decimal.fractional_digit_count() < 0 { decimal.with_scale(0) } else { decimal })
    }

    pub fn value(value: impl Display, decimals: u32) -> Result<String, NumberFormatterError> {
        Ok(Self::big_decimal_value(value, decimals)?.to_string())
    }

    pub fn plain_value(value: impl Display, decimals: u32) -> Result<String, NumberFormatterError> {
        Ok(Self::big_decimal_value(value, decimals)?.normalized().to_plain_string())
    }

    pub fn value_as_f64(value: impl Display, decimals: u32) -> Result<f64, NumberFormatterError> {
        Self::big_decimal_value(value, decimals)?.to_f64().ok_or_else(|| NumberFormatterError::ConversionError("Cannot convert to f64".to_string()))
    }

    pub fn f64_value(value: impl Display, decimals: u32) -> f64 {
        Self::value_as_f64(value, decimals).unwrap_or_default()
    }

    pub fn value_as_u64(value: impl Display, decimals: u32) -> Result<u64, NumberFormatterError> {
        Self::big_decimal_value(value, decimals)?.to_u64().ok_or_else(|| NumberFormatterError::ConversionError("Cannot convert to u64".to_string()))
    }

    fn amount_scaled(amount: impl Display, decimals: u32) -> Result<BigDecimal, NumberFormatterError> {
        let text = amount.to_string();
        let (digits, scale) = BigDecimal::from_str(&text).map_err(|_| NumberFormatterError::InvalidNumber(text))?.into_bigint_and_exponent();
        Ok(BigDecimal::new(digits, scale - i64::from(decimals)))
    }

    pub fn value_from_amount(amount: impl Display, decimals: u32) -> Result<String, NumberFormatterError> {
        Ok(Self::amount_scaled(amount, decimals)?.with_scale(0).to_string())
    }

    pub fn value_from_amount_truncated(amount: impl Display, decimals: u32) -> Result<String, NumberFormatterError> {
        let text = amount.to_string();
        let big_decimal = BigDecimal::from_str(&text).map_err(|_| NumberFormatterError::InvalidNumber(text.clone()))?;
        if big_decimal < 0 {
            return Err(NumberFormatterError::InvalidNumber(text));
        }
        Self::value_from_amount(big_decimal.with_scale_round(i64::from(decimals), RoundingMode::Down), decimals)
    }

    pub fn value_from_amount_exact(amount: impl Display, decimals: u32) -> Result<BigUint, NumberFormatterError> {
        let text = amount.to_string();
        let scaled_value = Self::amount_scaled(&text, decimals)?;
        if !scaled_value.is_integer() {
            return Err(NumberFormatterError::InvalidNumber(text));
        }
        let scaled_string = scaled_value.with_scale(0).to_string();
        scaled_string.parse::<BigUint>().map_err(|_| NumberFormatterError::ConversionError(scaled_string))
    }

    pub fn value_from_amount_biguint(amount: impl Display, decimals: u32) -> Result<BigUint, NumberFormatterError> {
        let scaled_string = Self::value_from_amount(amount, decimals)?;
        scaled_string.parse::<BigUint>().map_err(|_| NumberFormatterError::ConversionError(scaled_string))
    }

    pub fn decimal_to_string(value: &BigDecimal, max_scale: u32) -> String {
        value.round(max_scale as i64).normalized().to_string()
    }

    pub fn ratio(numerator: &BigUint, denominator: &BigUint) -> f64 {
        if *denominator == BigUint::from(0u32) {
            return 0.0;
        }
        let precision = BigUint::from(1_000_000u64);
        let scaled = numerator * &precision / denominator;
        let scaled_u64 = u64::try_from(&scaled).unwrap_or(u64::MAX);
        scaled_u64 as f64 / 1_000_000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bigdecimal::{BigDecimal, num_bigint::BigInt};
    use std::str::FromStr;

    #[test]
    fn test_f64_value() {
        assert_eq!(BigNumberFormatter::f64_value(BigInt::from(123456789), 8), 1.23456789);
        assert_eq!(BigNumberFormatter::f64_value(BigInt::from(0), 18), 0.0);
        assert_eq!(BigNumberFormatter::f64_value(BigInt::from(-2500), 3), -2.5);
        assert_eq!(BigNumberFormatter::f64_value(BigUint::from(1_000_000u32), 6), 1.0);

        let huge = BigInt::from(10).pow(400);
        assert!(BigNumberFormatter::f64_value(huge, 0).is_infinite(), "a value past f64 range saturates instead of failing");
    }

    #[test]
    fn test_plain_value_keeps_every_digit() {
        assert_eq!(BigNumberFormatter::plain_value(BigUint::from(1u32), 18).unwrap(), "0.000000000000000001");
        assert_eq!(BigNumberFormatter::plain_value(BigUint::from(1_500_000u32), 6).unwrap(), "1.5");
        assert_eq!(BigNumberFormatter::plain_value(BigUint::from(1_000u32), 0).unwrap(), "1000");
        assert_eq!(BigNumberFormatter::plain_value(&BigUint::ZERO, 18).unwrap(), "0");
        assert_eq!(BigNumberFormatter::plain_value(BigUint::from_str("123456789012345678901234567890").unwrap(), 18).unwrap(), "123456789012.34567890123456789");
    }

    #[test]
    fn test_value() {
        let result = BigNumberFormatter::value("123456", 3).unwrap();
        assert_eq!(result, "123.456");

        let result = BigNumberFormatter::value("789123456", 4).unwrap();
        assert_eq!(result, "78912.3456");

        let result = BigNumberFormatter::value("4567", 4).unwrap();
        assert_eq!(result, "0.4567");

        let result = BigNumberFormatter::value("115792089237316195423570985008687907853269984665640564039457000000000000000000", 18).unwrap();
        assert_eq!(result, "115792089237316195423570985008687907853269984665640564039457");

        let result = BigNumberFormatter::value("abc", 2);
        assert!(result.is_err());

        let result = BigNumberFormatter::value("1640000000000000", 18).unwrap();
        assert_eq!(result, "0.00164");
    }

    #[test]
    fn test_value_from_amount() {
        let result = BigNumberFormatter::value_from_amount("1.123", 3).unwrap();
        assert_eq!(result, "1123");

        let result = BigNumberFormatter::value_from_amount("332131212.2321312", 8).unwrap();
        assert_eq!(result, "33213121223213120");

        let result = BigNumberFormatter::value_from_amount("0", 0).unwrap();
        assert_eq!(result, "0");

        let result = BigNumberFormatter::value_from_amount("invalid", 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NumberFormatterError::InvalidNumber("invalid".to_string()));
    }

    #[test]
    fn test_value_from_amount_exact() {
        assert_eq!(BigNumberFormatter::value_from_amount_exact("1.123", 3).unwrap(), BigUint::from(1123u32));
        assert_eq!(BigNumberFormatter::value_from_amount_exact("0.0001", 8).unwrap(), BigUint::from(10000u32));
        assert_eq!(BigNumberFormatter::value_from_amount_exact("10", 6).unwrap(), BigUint::from(10_000_000u32));
        assert!(BigNumberFormatter::value_from_amount_exact("0.000000001", 8).is_err());
        assert!(BigNumberFormatter::value_from_amount_exact("-1", 8).is_err());
        assert!(BigNumberFormatter::value_from_amount_exact("invalid", 8).is_err());
    }

    #[test]
    fn test_value_from_amount_truncated() {
        assert_eq!(BigNumberFormatter::value_from_amount_truncated("1.183818719", 8).unwrap(), "118381871");
        assert_eq!(BigNumberFormatter::value_from_amount_truncated("123456789012345678.123456789", 18).unwrap(), "123456789012345678123456789000000000");
        assert_eq!(BigNumberFormatter::value_from_amount_truncated("-1", 9), Err(NumberFormatterError::InvalidNumber("-1".to_string())));
    }

    #[test]
    fn test_decimal_to_string() {
        let decimal = BigDecimal::from_str("1.123456789").unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&decimal, 6), "1.123457");

        let decimal = BigDecimal::from_str("0.000001234").unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&decimal, 6), "0.000001");

        let decimal = BigDecimal::from_str("2.5").unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&decimal, 6), "2.5");

        let decimal = BigDecimal::from_str("10").unwrap();
        assert_eq!(BigNumberFormatter::decimal_to_string(&decimal, 6), "10");
    }

    #[test]
    fn test_ratio() {
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(1u32), &BigUint::from(2u32)), 0.5);
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(1u32), &BigUint::from(4u32)), 0.25);
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(0u32), &BigUint::from(100u32)), 0.0);
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(100u32), &BigUint::from(0u32)), 0.0);
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(1u32), &BigUint::from(100u32)), 0.01);
        assert_eq!(BigNumberFormatter::ratio(&BigUint::from(3u32), &BigUint::from(100u32)), 0.03);

        let large_num = BigUint::from(1_000_000_000u64);
        let large_den = BigUint::from(10_000_000_000u64);
        assert_eq!(BigNumberFormatter::ratio(&large_num, &large_den), 0.1);
    }

    #[test]
    fn test_value_from_amount_biguint() {
        let result = BigNumberFormatter::value_from_amount_biguint("1.123", 3).unwrap();
        assert_eq!(result, BigUint::from(1123u32));

        let result = BigNumberFormatter::value_from_amount_biguint("332131212.2321312", 8).unwrap();
        assert_eq!(result, BigUint::from(33213121223213120_u64));

        let result = BigNumberFormatter::value_from_amount_biguint("0", 0).unwrap();
        assert_eq!(result, BigUint::from(0u32));

        let result = BigNumberFormatter::value_from_amount_biguint("1000000000000", 18).unwrap();
        let expected = "1000000000000000000000000000000".parse::<BigUint>().unwrap();
        assert_eq!(result, expected);

        let result = BigNumberFormatter::value_from_amount_biguint("invalid", 3);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), NumberFormatterError::InvalidNumber("invalid".to_string()));
    }
}
