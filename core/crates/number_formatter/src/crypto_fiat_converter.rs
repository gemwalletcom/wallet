use bigdecimal::{BigDecimal, RoundingMode};
use std::num::NonZeroU64;
use std::str::FromStr;

use crate::big_number_formatter::{BigNumberFormatter, NumberFormatterError};

const ENTRY_DUST_THRESHOLD: &str = "0.0001";

pub struct CryptoFiatConverter {}

impl CryptoFiatConverter {
    pub fn to_fiat(value: &str, decimals: u32, price: f64) -> Result<String, NumberFormatterError> {
        let amount = BigNumberFormatter::big_decimal_value(value, decimals)?;
        Ok((amount * Self::price_value(price)?).normalized().to_string())
    }

    pub fn to_crypto(fiat_amount: &str, decimals: u32, price: f64) -> Result<String, NumberFormatterError> {
        if price <= 0.0 {
            return Err(NumberFormatterError::InvalidNumber(format!("invalid price: {price}")));
        }
        let value = Self::fiat_value(fiat_amount)? / Self::price_value(price)?;
        Ok(BigNumberFormatter::decimal_to_string(&value, decimals))
    }

    pub fn to_crypto_at_entry_precision(fiat_amount: &str, decimals: u32, price: f64) -> Result<String, NumberFormatterError> {
        let value = Self::fiat_value(&Self::to_crypto(fiat_amount, decimals, price)?)?;
        Ok(Self::entry_precision(&value).normalized().to_plain_string())
    }

    fn entry_precision(value: &BigDecimal) -> BigDecimal {
        let magnitude = value.abs();
        if magnitude >= 1 {
            value.with_scale_round(2, RoundingMode::Down)
        } else if magnitude >= BigDecimal::from_str(ENTRY_DUST_THRESHOLD).expect("valid decimal") {
            value.with_precision_round(NonZeroU64::new(4).expect("non-zero"), RoundingMode::Down)
        } else {
            value.clone()
        }
    }

    fn fiat_value(fiat_amount: &str) -> Result<BigDecimal, NumberFormatterError> {
        BigDecimal::from_str(fiat_amount).map_err(|_| NumberFormatterError::InvalidNumber(fiat_amount.to_string()))
    }

    fn price_value(price: f64) -> Result<BigDecimal, NumberFormatterError> {
        BigDecimal::from_str(&price.to_string()).map_err(|_| NumberFormatterError::ConversionError(price.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fiat() {
        assert_eq!(CryptoFiatConverter::to_fiat("150000000", 8, 50_000.0).unwrap(), "75000");
        assert_eq!(CryptoFiatConverter::to_fiat("1000000000000000000", 18, 2500.5).unwrap(), "2500.5");
        assert_eq!(CryptoFiatConverter::to_fiat("1092000000000", 18, 3520.42).unwrap(), "0.00384429864");
        assert_eq!(CryptoFiatConverter::to_fiat("0", 8, 50_000.0).unwrap(), "0");
        assert_eq!(
            CryptoFiatConverter::to_fiat("123456789012345678901234567890", 18, 2.0).unwrap(),
            "246913578024.69135780246913578"
        );
        assert!(CryptoFiatConverter::to_fiat("abc", 8, 50_000.0).is_err());
    }

    #[test]
    fn test_to_crypto() {
        assert_eq!(CryptoFiatConverter::to_crypto("50000", 8, 50_000.0).unwrap(), "1");
        assert_eq!(CryptoFiatConverter::to_crypto("100", 8, 3.0).unwrap(), "33.33333333");
        assert_eq!(CryptoFiatConverter::to_crypto("0", 8, 50_000.0).unwrap(), "0");
        assert!(CryptoFiatConverter::to_crypto("abc", 8, 50_000.0).is_err());
    }

    #[test]
    fn test_to_crypto_at_entry_precision() {
        let at = |fiat: &str, decimals: u32, price: f64| CryptoFiatConverter::to_crypto_at_entry_precision(fiat, decimals, price).unwrap();
        assert_eq!(at("1", 8, 76_800.0), "0.00001302");
        assert_eq!(at("1", 8, 2.5), "0.4");
        assert_eq!(at("1", 8, 80.0), "0.0125");
        assert_eq!(at("1", 8, 8192.0), "0.000122");
        assert_eq!(at("10", 8, 2.5), "4");
        assert_eq!(at("10", 2, 3.33333333), "3");
        assert_eq!(at("1234", 6, 1000.0), "1.23");
        assert_eq!(at("1000.123456", 6, 1.0), "1000.12");
        assert_eq!(at("1000", 6, 1.0), "1000");
        assert_eq!(at("12345678", 6, 1.0), "12345678");
        assert_eq!(at("0.000000025", 8, 2.5), "0.00000001");
        assert_eq!(at("0", 8, 2.5), "0");
        assert!(CryptoFiatConverter::to_crypto_at_entry_precision("1", 18, 0.0).is_err());
    }

    #[test]
    fn test_to_crypto_fails_closed_on_invalid_price() {
        assert!(CryptoFiatConverter::to_crypto("100", 8, 0.0).is_err());
        assert!(CryptoFiatConverter::to_crypto("100", 8, -1.0).is_err());
        assert!(CryptoFiatConverter::to_crypto("100", 8, f64::NAN).is_err());
        assert!(CryptoFiatConverter::to_fiat("100", 8, f64::NAN).is_err());
    }
}
