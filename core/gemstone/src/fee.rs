use num_bigint::BigInt;
use primitives::{CustomFee, GasPriceType};

use crate::models::gateway::GemGasPriceType;

#[derive(uniffi::Object, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    fee_value: BigInt,
    max_rate: BigInt,
    is_over_max: bool,
}

#[uniffi::export]
impl GemCustomFee {
    #[uniffi::constructor]
    pub fn estimate(rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt, max_multiplier: u32) -> Self {
        let fee = CustomFee::calculate(rate, loaded_fee, base_total, normal_total, max_multiplier);

        Self {
            fee_value: fee.fee_value,
            max_rate: fee.max_rate,
            is_over_max: fee.is_over_max,
        }
    }

    pub fn fee_value(&self) -> BigInt {
        self.fee_value.clone()
    }

    pub fn max_rate(&self) -> BigInt {
        self.max_rate.clone()
    }

    pub fn is_over_max(&self) -> bool {
        self.is_over_max
    }
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
