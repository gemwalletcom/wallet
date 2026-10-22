use crate::constants::{BUILDER_ADDRESS, REFERRAL_CODE};
use crate::models::perp_dex::PerpDex;

#[derive(Debug, Clone, PartialEq)]
pub struct HypercoreConfig {
    pub builder_address: String,
    pub referral_code: String,
    pub max_builder_fee_bps: u32,
    pub perp_dexes: Vec<PerpDex>,
}

impl Default for HypercoreConfig {
    fn default() -> Self {
        Self {
            builder_address: BUILDER_ADDRESS.to_string(),
            referral_code: REFERRAL_CODE.to_string(),
            max_builder_fee_bps: 45,
            perp_dexes: vec![PerpDex::Main],
        }
    }
}
