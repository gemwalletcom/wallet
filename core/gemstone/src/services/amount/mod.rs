pub mod model;
pub mod rules;

use primitives::Asset;

use crate::services::transfer::GemTransferBalance;
pub use model::{GemAmountEarnType, GemAmountError, GemAmountLimits, GemAmountPerpetualPosition, GemAmountRules, GemAmountStakeType, GemAmountType};

#[derive(Default, uniffi::Object)]
pub struct GemAmountService;

#[uniffi::export]
impl GemAmountService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn rules(&self, amount_type: &GemAmountType, asset: Asset) -> GemAmountRules {
        rules::rules(amount_type, &asset)
    }

    pub fn limits(&self, amount_type: &GemAmountType, asset: Asset, balance: GemTransferBalance) -> Result<GemAmountLimits, GemAmountError> {
        rules::limits(amount_type, &asset, &balance)
    }

    pub fn validate(&self, value: String, available_value: String, minimum_value: String) -> Result<(), GemAmountError> {
        rules::validate(&rules::parse_value(&value)?, &rules::parse_value(&available_value)?, &rules::parse_value(&minimum_value)?)
    }
}
