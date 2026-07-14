use std::str::FromStr;

use num_bigint::BigInt;
use primitives::GasPriceType;

use crate::{GemstoneError, models::gateway::GemGasPriceType};

#[uniffi::export]
pub fn custom_gas_price(base: GemGasPriceType, gas_price: String) -> Result<GemGasPriceType, GemstoneError> {
    let base: GasPriceType = base.into();
    let gas_price = BigInt::from_str(&gas_price).map_err(|error| GemstoneError::from(error.to_string()))?;
    Ok(base.custom(gas_price).into())
}
