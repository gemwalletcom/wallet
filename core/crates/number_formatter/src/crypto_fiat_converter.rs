use bigdecimal::BigDecimal;
use std::str::FromStr;

use crate::big_number_formatter::{BigNumberFormatter, NumberFormatterError};

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

    fn fiat_value(fiat_amount: &str) -> Result<BigDecimal, NumberFormatterError> {
        BigDecimal::from_str(fiat_amount).map_err(|_| NumberFormatterError::InvalidNumber(fiat_amount.to_string()))
    }

    fn price_value(price: f64) -> Result<BigDecimal, NumberFormatterError> {
        BigDecimal::try_from(price).map_err(|_| NumberFormatterError::ConversionError(price.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_fiat() {
        assert_eq!(CryptoFiatConverter::to_fiat("150000000", 8, 50_000.0).unwrap(), "75000");
        assert_eq!(CryptoFiatConverter::to_fiat("1000000000000000000", 18, 2500.5).unwrap(), "2500.5");
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
    fn test_to_crypto_fails_closed_on_invalid_price() {
        assert!(CryptoFiatConverter::to_crypto("100", 8, 0.0).is_err());
        assert!(CryptoFiatConverter::to_crypto("100", 8, -1.0).is_err());
        assert!(CryptoFiatConverter::to_crypto("100", 8, f64::NAN).is_err());
        assert!(CryptoFiatConverter::to_fiat("100", 8, f64::NAN).is_err());
    }
}
