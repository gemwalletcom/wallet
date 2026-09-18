pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Release, Wallet};

use crate::models::list::GemListSection;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_session;

pub use rules::{GemPerpetualDefaults, GemPreferencesInput, GemSecurityInput};

#[derive(uniffi::Object)]
pub struct GemSettingsService {
    preferences: Arc<GemPreferencesService>,
}

#[uniffi::export]
impl GemSettingsService {
    #[uniffi::constructor]
    pub fn new(preferences: Arc<GemPreferencesService>) -> Self {
        Self { preferences }
    }

    pub fn preferences_sections(&self, input: GemPreferencesInput) -> Vec<GemListSection> {
        rules::preferences_sections(input)
    }

    pub fn perpetual_defaults(&self) -> GemPerpetualDefaults {
        GemPerpetualDefaults {
            leverage: self.preferences.get_perpetual_leverage(),
            take_profit_percent: self.preferences.get_perpetual_take_profit_percent(),
            stop_loss_percent: self.preferences.get_perpetual_stop_loss_percent(),
        }
    }

    pub fn set_perpetual_defaults(&self, defaults: GemPerpetualDefaults) -> Result<(), GemServiceError> {
        self.preferences.set_perpetual_leverage(defaults.leverage)?;
        self.preferences.set_perpetual_take_profit_percent(defaults.take_profit_percent)?;
        self.preferences.set_perpetual_stop_loss_percent(defaults.stop_loss_percent)
    }

    pub fn security_sections(&self, input: GemSecurityInput) -> Vec<GemListSection> {
        rules::security_sections(input)
    }

    pub fn sections(&self, wallets: Vec<Wallet>, notifications_available: bool, wallet_connect_available: bool) -> Vec<GemListSection> {
        rules::sections(
            wallets.len(),
            notifications_available,
            wallet_connect_available,
            wallet_session::rules::shows_rewards(&wallets),
            self.preferences.is_developer_enabled(),
        )
    }
}

#[uniffi::export]
pub fn about_sections(version: String, update: Option<Release>) -> Vec<GemListSection> {
    rules::about_sections(version, update)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_perpetual_defaults_the_screen_shows_are_the_ones_it_last_wrote() {
        let service = GemSettingsService::mock();
        let written = GemPerpetualDefaults {
            leverage: 7,
            take_profit_percent: 30,
            stop_loss_percent: 12,
        };

        service.set_perpetual_defaults(written).unwrap();

        assert_eq!(service.perpetual_defaults(), written);
    }
}
