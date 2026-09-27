pub mod rules;
#[cfg(test)]
pub(crate) mod testkit;

use std::sync::Arc;

use primitives::{Release, Wallet};

use crate::models::list::GemListSection;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::preferences::GemPreferencesService;
use crate::services::wallet_session;

pub use rules::{GemAboutViewState, GemPerpetualDefaults, GemPerpetualPickers, GemPickerOption, GemSecurityInput};

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

    pub fn preferences_sections(&self, language: Option<String>, appearance: String) -> Vec<GemListSection> {
        rules::preferences_sections(rules::PreferencesInput {
            currency: self.preferences.get_currency(),
            language,
            appearance,
            perpetuals_enabled: self.preferences.is_perpetual_enabled(),
            perpetual_defaults: self.perpetual_defaults(),
        })
    }

    pub fn perpetual_pickers(&self) -> GemPerpetualPickers {
        rules::perpetual_pickers()
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
        rules::security_sections(input, self.preferences.is_hide_balance_enabled())
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
pub fn notifications_sections(push_enabled: bool) -> Vec<GemListSection> {
    rules::notifications_sections(push_enabled)
}

#[uniffi::export]
pub fn about_view_state(version: String, build: String, update: Option<Release>, developer_enabled: bool) -> GemAboutViewState {
    GemAboutViewState {
        sections: rules::about_sections(version, build, update),
        developer_toggle: match developer_enabled {
            true => GemLocalizedText::DisableDeveloper,
            false => GemLocalizedText::EnableDeveloper,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_about_view_state_names_the_developer_toggle_from_the_flag() {
        assert_eq!(about_view_state("1".to_string(), "2".to_string(), None, false).developer_toggle, GemLocalizedText::EnableDeveloper);
        assert_eq!(about_view_state("1".to_string(), "2".to_string(), None, true).developer_toggle, GemLocalizedText::DisableDeveloper);
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

        assert_eq!(service.perpetual_defaults(), written);
    }

    #[test]
    fn test_the_settings_screens_show_what_the_preference_store_holds() {
        use crate::models::list::{GemListRow, GemListRowTitle, GemRowAction};

        let service = GemSettingsService::mock();
        let toggle_on = |sections: Vec<GemListSection>, action: GemRowAction| {
            sections.iter().flat_map(|section| section.rows.clone()).find_map(|row| match row {
                GemListRow::Toggle { is_on, action: row_action, .. } if row_action == action => Some(is_on),
                _ => None,
            })
        };
        let security = || {
            service.security_sections(GemSecurityInput {
                authentication_enabled: false,
                authentication_name: None,
                lock_period: String::new(),
                privacy_lock_enabled: false,
                privacy_lock_supported: false,
            })
        };

        assert_eq!(toggle_on(service.preferences_sections(None, String::new()), GemRowAction::Perpetuals), Some(false));
        assert_eq!(toggle_on(security(), GemRowAction::HideBalance), Some(false));

        service.preferences.set_perpetual_enabled(true).unwrap();
        service.preferences.set_hide_balance_enabled(true).unwrap();

        let preferences = service.preferences_sections(None, String::new());
        assert_eq!(toggle_on(preferences.clone(), GemRowAction::Perpetuals), Some(true));
        assert!(
            preferences.iter().flat_map(|section| section.rows.iter()).any(|row| matches!(
                row,
                GemListRow::Picker {
                    title: GemListRowTitle::PerpetualLeverage,
                    ..
                }
            )),
            "the stored switch opens the perpetual defaults"
        );
        assert_eq!(toggle_on(security(), GemRowAction::HideBalance), Some(true));
    }
}
