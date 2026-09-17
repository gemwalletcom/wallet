use num_bigint::BigInt;
use primitives::{Chain, CustomFee};

use crate::config::fee_config::get_fee_config;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemCustomFeeCheck {
    Valid,
    BelowMinimum,
    OverMaximum,
}

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

    pub fn check(&self) -> GemCustomFeeCheck {
        match (self.fee.is_below_minimum, self.fee.is_over_max) {
            (true, _) => GemCustomFeeCheck::BelowMinimum,
            (false, true) => GemCustomFeeCheck::OverMaximum,
            (false, false) => GemCustomFeeCheck::Valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.fee.is_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn custom_fee(is_below_minimum: bool, is_over_max: bool) -> GemCustomFee {
        GemCustomFee {
            fee: CustomFee {
                fee_value: BigInt::from(1),
                max_rate: BigInt::from(20),
                minimum_rate: Some(BigInt::from(5)),
                is_over_max,
                is_below_minimum,
                is_valid: !is_below_minimum && !is_over_max,
            },
        }
    }

    #[test]
    fn test_the_check_names_the_bound_a_custom_rate_breaks() {
        assert_eq!(custom_fee(false, false).check(), GemCustomFeeCheck::Valid);
        assert_eq!(custom_fee(true, false).check(), GemCustomFeeCheck::BelowMinimum);
        assert_eq!(custom_fee(false, true).check(), GemCustomFeeCheck::OverMaximum);
    }
}
