use num_bigint::BigInt;
use primitives::Chain;

use crate::config::chain::minimum_custom_fee_rate;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    pub max_multiplier: u32,
    pub minimum_custom_fee_rate: Option<BigInt>,
}

pub fn get_fee_config(chain: Chain) -> FeeConfig {
    FeeConfig {
        max_multiplier: 10,
        minimum_custom_fee_rate: minimum_custom_fee_rate(chain),
    }
}
