use std::str::FromStr;

use num_bigint::BigInt;
use primitives::{CustomFee, GasPriceType};

use crate::{GemstoneError, models::gateway::GemGasPriceType};

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    pub fee_amount: String,
    pub max_rate: String,
    pub is_over_max: bool,
}

#[uniffi::export]
pub fn custom_gas_price(base: GemGasPriceType, gas_price: String) -> Result<GemGasPriceType, GemstoneError> {
    let base: GasPriceType = base.into();
    let gas_price = BigInt::from_str(&gas_price).map_err(|error| GemstoneError::from(error.to_string()))?;
    Ok(base.custom(gas_price).into())
}

#[uniffi::export]
pub fn custom_fee_estimate(rate: Option<String>, loaded_fee: String, base_total: String, normal_total: String, max_multiplier: u32) -> Result<GemCustomFee, GemstoneError> {
    let parse = |value: &str| BigInt::from_str(value).map_err(|error| GemstoneError::from(error.to_string()));
    let rate = rate.map(|value| parse(&value)).transpose()?;

    let fee = CustomFee::calculate(rate, parse(&loaded_fee)?, parse(&base_total)?, parse(&normal_total)?, max_multiplier);

    Ok(GemCustomFee {
        fee_amount: fee.fee_amount.to_string(),
        max_rate: fee.max_rate.to_string(),
        is_over_max: fee.is_over_max,
    })
}
