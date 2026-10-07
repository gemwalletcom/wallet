use crate::constants::{BUILDER_ADDRESS, REFERRAL_CODE};

#[derive(Debug, Clone, PartialEq)]
pub struct HypercoreConfig {
    pub builder_address: String,
    pub referral_code: String,
    pub max_builder_fee_bps: u32,
    pub enabled_hip3_markets: Vec<String>,
}

impl Default for HypercoreConfig {
    fn default() -> Self {
        Self {
            builder_address: BUILDER_ADDRESS.to_string(),
            referral_code: REFERRAL_CODE.to_string(),
            max_builder_fee_bps: 45,
            enabled_hip3_markets: vec![],
        }
    }
}
