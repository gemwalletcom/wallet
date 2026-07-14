use primitives::Chain;

use crate::config::chain::fee_unit_type;

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    pub unit_decimals: u32,
    pub max_custom_fee_rate_multiplier: u32,
}

pub fn get_fee_config(chain: Chain) -> FeeConfig {
    FeeConfig {
        unit_decimals: fee_unit_type(chain).decimals(),
        max_custom_fee_rate_multiplier: 10,
    }
}
