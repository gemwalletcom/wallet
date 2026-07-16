use primitives::Chain;

use crate::config::chain::{custom_fee_enabled, fee_unit_type};

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct FeeConfig {
    pub decimals: u32,
    pub max_multiplier: u32,
    pub custom_fee_enabled: bool,
}

pub fn get_fee_config(chain: Chain) -> FeeConfig {
    FeeConfig {
        decimals: fee_unit_type(chain).decimals(),
        max_multiplier: 10,
        custom_fee_enabled: custom_fee_enabled(chain),
    }
}
