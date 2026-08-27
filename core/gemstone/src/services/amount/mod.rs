pub mod model;
pub mod rules;

use num_bigint::BigInt;
use primitives::Asset;

pub use model::{GemAmountBalance, GemAmountEarnType, GemAmountError, GemAmountLimits, GemAmountPerpetualPosition, GemAmountRules, GemAmountStakeType, GemAmountType};

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

    pub fn limits(&self, amount_type: &GemAmountType, asset: Asset, balance: GemAmountBalance) -> GemAmountLimits {
        rules::limits(amount_type, &asset, &balance)
    }

    pub fn validate(&self, value: String, available_value: String, minimum_value: String) -> Result<(), GemAmountError> {
        let parse = |value: &str| value.parse::<BigInt>().unwrap_or_default();
        rules::validate(&parse(&value), &parse(&available_value), &parse(&minimum_value))
    }
}
