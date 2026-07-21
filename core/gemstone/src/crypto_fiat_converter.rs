use num_bigint::BigInt;
use number_formatter::CryptoFiatConverter as NumberFormatterCryptoFiatConverter;

use crate::GemstoneError;

#[derive(Debug, uniffi::Object)]
pub struct CryptoFiatConverter {}

impl Default for CryptoFiatConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl CryptoFiatConverter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn convert_to_fiat(&self, value: BigInt, decimals: u32, price: f64) -> Result<String, GemstoneError> {
        Ok(NumberFormatterCryptoFiatConverter::convert_to_fiat(&value.to_string(), decimals, price)?)
    }

    pub fn convert_to_crypto(&self, fiat_amount: String, decimals: u32, price: f64) -> Result<String, GemstoneError> {
        Ok(NumberFormatterCryptoFiatConverter::convert_to_crypto(&fiat_amount, decimals, price)?)
    }
}
