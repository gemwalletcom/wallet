use primitives::Chain;

use crate::config::chain::{custom_fee_enabled, minimum_custom_fee_rate};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    pub decimals: u32,
    pub max_multiplier: u32,
    pub custom_fee_enabled: bool,
    pub minimum_custom_fee_rate: Option<u32>,
}

pub fn get_fee_config(chain: Chain) -> FeeConfig {
    FeeConfig {
        decimals: chain.fee_unit_type().decimals(),
        max_multiplier: 10,
        custom_fee_enabled: custom_fee_enabled(chain),
        minimum_custom_fee_rate: minimum_custom_fee_rate(chain),
    }
}
