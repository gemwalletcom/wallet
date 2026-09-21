use num_bigint::BigInt;
use number_formatter::CryptoFiatConverter as Converter;

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

    pub fn to_fiat(&self, value: BigInt, decimals: u32, price: f64) -> f64 {
        Converter::fiat_f64(&value.to_string(), decimals, price)
    }
}
