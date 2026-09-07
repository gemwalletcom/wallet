use num_bigint::BigInt;
use primitives::{Chain, CustomFee};

use crate::config::fee_config::get_fee_config;

#[derive(uniffi::Object, Clone, Debug, PartialEq, Eq)]
pub struct GemCustomFee {
    fee: CustomFee,
}

#[uniffi::export]
impl GemCustomFee {
    #[uniffi::constructor]
    pub fn estimate(chain: Chain, rate: Option<BigInt>, loaded_fee: BigInt, base_total: BigInt, normal_total: BigInt) -> Self {
        let config = get_fee_config(chain);
        Self {
            fee: CustomFee::calculate(
                rate,
                loaded_fee,
                base_total,
                normal_total,
                config.max_multiplier,
                config.minimum_custom_fee_rate.map(BigInt::from),
            ),
        }
    }

    pub fn fee_value(&self) -> BigInt {
        self.fee.fee_value.clone()
    }

    pub fn max_rate(&self) -> BigInt {
        self.fee.max_rate.clone()
    }

    pub fn minimum_rate(&self) -> Option<BigInt> {
        self.fee.minimum_rate.clone()
    }

    pub fn is_over_max(&self) -> bool {
        self.fee.is_over_max
    }

    pub fn is_below_minimum(&self) -> bool {
        self.fee.is_below_minimum
    }

    pub fn is_valid(&self) -> bool {
        self.fee.is_valid
    }
}
