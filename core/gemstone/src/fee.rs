use num_bigint::BigInt;
use primitives::{CustomFee, GasPriceType};

use crate::models::gateway::GemGasPriceType;

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    pub fee_value: BigInt,
    pub max_rate: BigInt,
    pub is_over_max: bool,
}

impl GemGasPriceType {
    pub(crate) fn custom_gas_price(&self, gas_price: BigInt) -> Self {
        GasPriceType::from(self.clone()).custom(gas_price).into()
    }
}

#[uniffi::export]
impl GemGasPriceType {
    pub fn total_fee(&self) -> BigInt {
        GasPriceType::from(self.clone()).total_fee()
    }
}

#[derive(Default, uniffi::Object)]
pub struct GemFeeService {}

#[uniffi::export]
impl GemFeeService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn custom_fee_estimate(&self, rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt, max_multiplier: u32) -> GemCustomFee {
        let fee = CustomFee::calculate(rate, loaded_fee, base_total, normal_total, max_multiplier);

        GemCustomFee {
            fee_value: fee.fee_value,
            max_rate: fee.max_rate,
            is_over_max: fee.is_over_max,
        }
    }
}
