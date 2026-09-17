use crate::services::currency::GemCurrencyRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSettingsRow {
    Wallets,
    Security,
    Notifications,
    Preferences,
    WalletConnect,
    Support,
    Rewards,
    AboutUs,
    Developer,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSettingsSection {
    pub rows: Vec<GemSettingsRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPreferencesRow {
    Currency,
    Language,
    Appearance,
    Networks,
    Contacts,
    Perpetuals,
    PerpetualLeverage,
    PerpetualTakeProfit,
    PerpetualStopLoss,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemPreferencesSection {
    pub rows: Vec<GemPreferencesRow>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPreferencesState {
    pub currency: GemCurrencyRow,
    pub sections: Vec<GemPreferencesSection>,
    pub perpetual_defaults: GemPerpetualDefaults,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemPerpetualDefaults {
    pub leverage: u8,
    pub take_profit_percent: u8,
    pub stop_loss_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSecurityRow {
    Authentication,
    LockPeriod,
    PrivacyLock,
    HideBalance,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSecuritySection {
    pub rows: Vec<GemSecurityRow>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAboutRow {
    TermsOfService,
    PrivacyPolicy,
    Website,
    Community,
    Version,
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemAboutSection {
    pub rows: Vec<GemAboutRow>,
}

pub fn preferences_sections(perpetuals_enabled: bool) -> Vec<GemPreferencesSection> {
    [
        vec![
            GemPreferencesRow::Currency,
            GemPreferencesRow::Language,
            GemPreferencesRow::Appearance,
            GemPreferencesRow::Networks,
            GemPreferencesRow::Contacts,
        ],
        [
            Some(GemPreferencesRow::Perpetuals),
            perpetuals_enabled.then_some(GemPreferencesRow::PerpetualLeverage),
            perpetuals_enabled.then_some(GemPreferencesRow::PerpetualTakeProfit),
            perpetuals_enabled.then_some(GemPreferencesRow::PerpetualStopLoss),
        ]
        .into_iter()
        .flatten()
        .collect(),
    ]
    .into_iter()
    .map(|rows| GemPreferencesSection { rows })
    .collect()
}

pub fn security_sections(authentication_enabled: bool) -> Vec<GemSecuritySection> {
    [
        [
            Some(GemSecurityRow::Authentication),
            authentication_enabled.then_some(GemSecurityRow::LockPeriod),
            authentication_enabled.then_some(GemSecurityRow::PrivacyLock),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>(),
        vec![GemSecurityRow::HideBalance],
    ]
    .into_iter()
    .map(|rows| GemSecuritySection { rows })
    .collect()
}

pub fn about_sections() -> Vec<GemAboutSection> {
    [
        vec![GemAboutRow::TermsOfService, GemAboutRow::PrivacyPolicy, GemAboutRow::Website],
        vec![GemAboutRow::Community],
        vec![GemAboutRow::Version],
    ]
    .into_iter()
    .map(|rows| GemAboutSection { rows })
    .collect()
}

pub fn sections(notifications_available: bool, wallet_connect_available: bool, shows_rewards: bool, developer_enabled: bool) -> Vec<GemSettingsSection> {
    [
        vec![GemSettingsRow::Wallets, GemSettingsRow::Security],
        [notifications_available.then_some(GemSettingsRow::Notifications), Some(GemSettingsRow::Preferences)]
            .into_iter()
            .flatten()
            .collect(),
        wallet_connect_available.then_some(vec![GemSettingsRow::WalletConnect]).unwrap_or_default(),
        [
            Some(GemSettingsRow::Support),
            shows_rewards.then_some(GemSettingsRow::Rewards),
            Some(GemSettingsRow::AboutUs),
            developer_enabled.then_some(GemSettingsRow::Developer),
        ]
        .into_iter()
        .flatten()
        .collect(),
    ]
    .into_iter()
    .filter(|rows: &Vec<GemSettingsRow>| !rows.is_empty())
    .map(|rows| GemSettingsSection { rows })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_perpetual_defaults_show_only_once_perpetuals_are_on() {
        assert_eq!(
            preferences_sections(false).last().map(|section| section.rows.clone()),
            Some(vec![GemPreferencesRow::Perpetuals])
        );
        assert_eq!(preferences_sections(true).last().map(|section| section.rows.len()), Some(4));
    }

    #[test]
    fn test_the_lock_rows_show_only_once_authentication_is_on() {
        assert_eq!(
            security_sections(false).first().map(|section| section.rows.clone()),
            Some(vec![GemSecurityRow::Authentication])
        );
        assert_eq!(
            security_sections(true).first().map(|section| section.rows.clone()),
            Some(vec![GemSecurityRow::Authentication, GemSecurityRow::LockPeriod, GemSecurityRow::PrivacyLock])
        );
        assert_eq!(
            security_sections(true).last().map(|section| section.rows.clone()),
            Some(vec![GemSecurityRow::HideBalance]),
            "hiding the balance is its own choice, not part of the lock"
        );
    }

    #[test]
    fn test_the_settings_rows_follow_what_the_device_and_wallet_offer() {
        let full = sections(true, true, true, true);
        assert_eq!(
            full.iter().map(|section| section.rows.clone()).collect::<Vec<_>>(),
            vec![
                vec![GemSettingsRow::Wallets, GemSettingsRow::Security],
                vec![GemSettingsRow::Notifications, GemSettingsRow::Preferences],
                vec![GemSettingsRow::WalletConnect],
                vec![GemSettingsRow::Support, GemSettingsRow::Rewards, GemSettingsRow::AboutUs, GemSettingsRow::Developer],
            ]
        );

        let plain = sections(false, false, false, false);
        assert_eq!(
            plain.iter().map(|section| section.rows.clone()).collect::<Vec<_>>(),
            vec![
                vec![GemSettingsRow::Wallets, GemSettingsRow::Security],
                vec![GemSettingsRow::Preferences],
                vec![GemSettingsRow::Support, GemSettingsRow::AboutUs],
            ],
            "a device without notifications or WalletConnect drops those rows and their empty section"
        );
    }
}
