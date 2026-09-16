pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Currency, Wallet};

use crate::services::currency;
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_session;

pub use rules::{
    GemAboutRow, GemAboutSection, GemPerpetualDefaults, GemPreferencesRow, GemPreferencesSection, GemPreferencesState, GemSecurityRow, GemSecuritySection, GemSettingsRow,
    GemSettingsSection,
};

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

    pub fn preferences(&self, currency: Currency, perpetuals_enabled: bool) -> GemPreferencesState {
        GemPreferencesState {
            currency: currency::rules::row(currency),
            sections: rules::preferences_sections(perpetuals_enabled),
            perpetual_defaults: GemPerpetualDefaults {
                leverage: self.preferences.get_perpetual_leverage(),
                take_profit_percent: self.preferences.get_perpetual_take_profit_percent(),
                stop_loss_percent: self.preferences.get_perpetual_stop_loss_percent(),
            },
        }
    }

    pub fn set_perpetual_defaults(&self, defaults: GemPerpetualDefaults) -> Result<(), GemServiceError> {
        self.preferences.set_perpetual_leverage(defaults.leverage)?;
        self.preferences.set_perpetual_take_profit_percent(defaults.take_profit_percent)?;
        self.preferences.set_perpetual_stop_loss_percent(defaults.stop_loss_percent)
    }

    pub fn security_sections(&self, authentication_enabled: bool) -> Vec<GemSecuritySection> {
        rules::security_sections(authentication_enabled)
    }

    pub fn sections(&self, wallets: Vec<Wallet>, notifications_available: bool, wallet_connect_available: bool) -> Vec<GemSettingsSection> {
        rules::sections(
            notifications_available,
            wallet_connect_available,
            wallet_session::rules::shows_rewards(&wallets),
            self.preferences.is_developer_enabled(),
        )
    }
}

#[uniffi::export]
pub fn about_sections() -> Vec<GemAboutSection> {
    rules::about_sections()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_preferences_screen_flags_the_selected_currency_beside_its_rows() {
        let state = GemSettingsService::mock().preferences(Currency::GBP, false);

        assert_eq!(state.currency.text(), "\u{1f1ec}\u{1f1e7} GBP");
        assert_eq!(state.sections, rules::preferences_sections(false));
    }

    #[test]
    fn test_the_perpetual_defaults_the_screen_shows_are_the_ones_it_last_wrote() {
        let service = GemSettingsService::mock();
        let written = GemPerpetualDefaults {
            leverage: 7,
            take_profit_percent: 30,
            stop_loss_percent: 12,
        };

        service.set_perpetual_defaults(written).unwrap();

        assert_eq!(service.preferences(Currency::USD, true).perpetual_defaults, written);
    }
}
