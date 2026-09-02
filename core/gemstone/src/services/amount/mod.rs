pub mod model;
pub mod rules;

use std::sync::Arc;

use primitives::{AssetId, Currency};

pub use model::{GemAmountEarnType, GemAmountError, GemAmountLimits, GemAmountPerpetualPosition, GemAmountRules, GemAmountStakeType, GemAmountType};

use crate::models::{GemContractCallData, GemEarnType};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::stake::GemStakeService;

#[derive(uniffi::Object)]
pub struct GemAmountService {
    stake: Arc<GemStakeService>,
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemAmountService {
    #[uniffi::constructor]
    pub fn new(stake: Arc<GemStakeService>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { stake, preferences }
    }

    pub fn currency(&self) -> Currency {
        self.preferences.get_currency()
    }

    pub fn perpetual_leverage(&self) -> u8 {
        self.preferences.get_perpetual_leverage()
    }

    pub fn perpetual_take_profit_percent(&self) -> u8 {
        self.preferences.get_perpetual_take_profit_percent()
    }

    pub fn perpetual_stop_loss_percent(&self) -> u8 {
        self.preferences.get_perpetual_stop_loss_percent()
    }

    pub async fn earn_data(&self, asset_id: AssetId, address: String, value: String, earn_type: GemEarnType) -> Result<GemContractCallData, GemServiceError> {
        self.stake.get_earn_data(asset_id, address, value, earn_type).await
    }
}
