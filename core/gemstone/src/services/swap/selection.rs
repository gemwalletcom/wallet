use primitives::AssetId;

use super::model::GemSwapPairSuggestion;
use super::rules;

#[derive(Default, uniffi::Object)]
pub struct GemSwapSelectionService {}

#[uniffi::export]
impl GemSwapSelectionService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn pair_for_asset(&self, asset_id: AssetId, has_balance: bool) -> GemSwapPairSuggestion {
        rules::pair_for_asset(asset_id, has_balance)
    }
}
