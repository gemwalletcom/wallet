use num_bigint::BigInt;
use primitives::{CustomFee, GasPriceType};

use crate::models::gateway::GemGasPriceType;

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    pub fee_value: BigInt,
    pub max_rate: BigInt,
    pub is_over_max: bool,
}

#[uniffi::export]
pub fn custom_gas_price(base: GemGasPriceType, gas_price: BigInt) -> GemGasPriceType {
    let base: GasPriceType = base.into();
    base.custom(gas_price).into()
}

#[uniffi::export]
pub fn custom_fee_estimate(rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt, max_multiplier: u32) -> GemCustomFee {
    let fee = CustomFee::calculate(rate, loaded_fee, base_total, normal_total, max_multiplier);

    GemCustomFee {
        fee_value: fee.fee_value,
        max_rate: fee.max_rate,
        is_over_max: fee.is_over_max,
    }
}
