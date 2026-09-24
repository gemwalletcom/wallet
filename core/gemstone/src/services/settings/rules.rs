use primitives::{Currency, PlatformStore, Release};

use crate::config::perpetual_config;
use crate::config::public::PublicUrl;
use crate::config::social::community_links;
use crate::formatted_number::GemFormattedNumber;
use crate::models::list::{GemListRow, GemListRowIcon, GemListRowTitle, GemListSection, GemListSectionFooter, GemListSectionTitle, GemUrlTarget};
use crate::percentage::GemPercentageStyle;
use crate::services::currency;
use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemPerpetualDefaults {
    pub leverage: u8,
    pub take_profit_percent: u8,
    pub stop_loss_percent: u8,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPreferencesInput {
    pub currency: Currency,
    pub language: Option<String>,
    pub appearance: String,
    pub perpetuals_enabled: bool,
    pub perpetual_defaults: GemPerpetualDefaults,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPickerOption {
    pub value: u8,
    pub label: GemLocalizedText,
}

pub fn leverage_option(value: u8) -> GemPickerOption {
    GemPickerOption {
        value,
        label: GemLocalizedText::Number {
            number: GemFormattedNumber::leverage(value as f64),
        },
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemPerpetualPickers {
    pub leverage: Vec<GemPickerOption>,
    pub take_profit: Vec<GemPickerOption>,
    pub stop_loss: Vec<GemPickerOption>,
}

pub fn perpetual_pickers() -> GemPerpetualPickers {
    GemPerpetualPickers {
        leverage: perpetual_config::LEVERAGE_OPTIONS.iter().map(|value| leverage_option(*value)).collect(),
        take_profit: autoclose_options(perpetual_config::TAKE_PROFIT_PERCENT_OPTIONS),
        stop_loss: autoclose_options(perpetual_config::STOP_LOSS_PERCENT_OPTIONS),
    }
}

fn autoclose_options(values: &[u8]) -> Vec<GemPickerOption> {
    values
        .iter()
        .map(|value| GemPickerOption {
            value: *value,
            label: autoclose_label(*value),
        })
        .collect()
}

fn autoclose_label(percent: u8) -> GemLocalizedText {
    match percent {
        0 => GemLocalizedText::None,
        percent => GemLocalizedText::Number {
            number: GemFormattedNumber::percentage(percent as f64, GemPercentageStyle::UnsignedCompact),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Record)]
pub struct GemSecurityInput {
    pub authentication_enabled: bool,
    pub authentication_name: Option<String>,
    pub lock_period: String,
    pub privacy_lock_enabled: bool,
    pub privacy_lock_supported: bool,
    pub hide_balance_enabled: bool,
}

pub fn preferences_sections(input: GemPreferencesInput) -> Vec<GemListSection> {
    let link = |title: GemListRowTitle, value: Option<String>, icon: GemListRowIcon| GemListRow::Link { title, value, icon };
    let picker = |title: GemListRowTitle, value: GemLocalizedText| GemListRow::Picker { title, value, icon: GemListRowIcon::None };
    let pickers = perpetual_pickers();
    let label = |options: &[GemPickerOption], value: u8| options.iter().find(|option| option.value == value).map(|option| option.label.clone()).unwrap_or(GemLocalizedText::None);
    let section = |rows: Vec<GemListRow>| GemListSection {
        title: GemListSectionTitle::None,
        footer: GemListSectionFooter::None,
        rows,
    };
    vec![
        section(
            [
                Some(link(GemListRowTitle::Currency, Some(currency::rules::currency_text(&input.currency)), GemListRowIcon::Currency)),
                input.language.map(|language| link(GemListRowTitle::Language, Some(language), GemListRowIcon::Language)),
                Some(link(GemListRowTitle::Appearance, Some(input.appearance), GemListRowIcon::Appearance)),
                Some(link(GemListRowTitle::Networks, None, GemListRowIcon::Networks)),
                Some(link(GemListRowTitle::Contacts, None, GemListRowIcon::Contacts)),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
        section(
            [
                Some(GemListRow::Toggle {
                    title: GemListRowTitle::Perpetuals,
                    value: None,
                    icon: GemListRowIcon::Perpetuals,
                    is_on: input.perpetuals_enabled,
                }),
                input.perpetuals_enabled.then(|| picker(GemListRowTitle::PerpetualLeverage, label(&pickers.leverage, input.perpetual_defaults.leverage))),
                input
                    .perpetuals_enabled
                    .then(|| picker(GemListRowTitle::PerpetualTakeProfit, label(&pickers.take_profit, input.perpetual_defaults.take_profit_percent))),
                input
                    .perpetuals_enabled
                    .then(|| picker(GemListRowTitle::PerpetualStopLoss, label(&pickers.stop_loss, input.perpetual_defaults.stop_loss_percent))),
            ]
            .into_iter()
            .flatten()
            .collect(),
        ),
    ]
}

pub fn notifications_sections(push_enabled: bool) -> Vec<GemListSection> {
    let section = |row: GemListRow| GemListSection {
        title: GemListSectionTitle::None,
        footer: GemListSectionFooter::None,
        rows: vec![row],
    };
    vec![
        section(GemListRow::Toggle {
            title: GemListRowTitle::Notifications,
            value: None,
            icon: GemListRowIcon::None,
            is_on: push_enabled,
        }),
        section(GemListRow::Link {
            title: GemListRowTitle::PriceAlerts,
            value: None,
            icon: GemListRowIcon::PriceAlerts,
        }),
    ]
}

pub fn security_sections(input: GemSecurityInput) -> Vec<GemListSection> {
    let toggle = |title: GemListRowTitle, is_on: bool| GemListRow::Toggle {
        title,
        value: None,
        icon: GemListRowIcon::None,
        is_on,
    };
    vec![
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::Authentication,
            rows: [
                Some(GemListRow::Toggle {
                    title: GemListRowTitle::Authentication,
                    value: input.authentication_name,
                    icon: GemListRowIcon::None,
                    is_on: input.authentication_enabled,
                }),
                input.authentication_enabled.then_some(GemListRow::Picker {
                    title: GemListRowTitle::LockPeriod,
                    value: GemLocalizedText::Text { text: input.lock_period },
                    icon: GemListRowIcon::None,
                }),
                (input.authentication_enabled && input.privacy_lock_supported).then(|| toggle(GemListRowTitle::PrivacyLock, input.privacy_lock_enabled)),
            ]
            .into_iter()
            .flatten()
            .collect(),
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![toggle(GemListRowTitle::HideBalance, input.hide_balance_enabled)],
        },
    ]
}

pub fn about_sections(version: String, build: String, update: Option<Release>) -> Vec<GemListSection> {
    let page = |title: GemListRowTitle, url: PublicUrl| GemListRow::Url {
        title,
        value: None,
        icon: GemListRowIcon::None,
        url: url.url(),
        target: GemUrlTarget::InApp,
    };
    vec![
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: vec![
                page(GemListRowTitle::TermsOfService, PublicUrl::TermsOfService),
                page(GemListRowTitle::PrivacyPolicy, PublicUrl::PrivacyPolicy),
                page(GemListRowTitle::Website, PublicUrl::Website),
            ],
        },
        GemListSection {
            title: GemListSectionTitle::Community,
            footer: GemListSectionFooter::None,
            rows: vec![GemListRow::Social { links: community_links() }],
        },
        GemListSection {
            title: GemListSectionTitle::None,
            footer: GemListSectionFooter::None,
            rows: [
                Some(GemListRow::Text {
                    title: GemListRowTitle::Version,
                    value: format!("{version} ({build})"),
                }),
                update.map(|release| GemListRow::Url {
                    title: GemListRowTitle::UpdateApp,
                    value: Some(release.version),
                    icon: GemListRowIcon::AppLogo,
                    url: store_url(release.store).url(),
                    target: GemUrlTarget::External,
                }),
            ]
            .into_iter()
            .flatten()
            .collect(),
        },
    ]
}

fn store_url(store: PlatformStore) -> PublicUrl {
    match store {
        PlatformStore::AppStore => PublicUrl::AppStore,
        PlatformStore::GooglePlay => PublicUrl::PlayStore,
        PlatformStore::Fdroid | PlatformStore::Huawei | PlatformStore::SolanaStore | PlatformStore::SamsungStore | PlatformStore::ApkUniversal | PlatformStore::Emerald | PlatformStore::Local => PublicUrl::APK,
    }
}

pub fn sections(wallets_count: usize, notifications_available: bool, wallet_connect_available: bool, shows_rewards: bool, developer_enabled: bool) -> Vec<GemListSection> {
    let link = |title: GemListRowTitle, icon: GemListRowIcon| GemListRow::Link { title, value: None, icon };
    [
        vec![
            GemListRow::Link {
                title: GemListRowTitle::Wallets,
                value: Some(wallets_count.to_string()),
                icon: GemListRowIcon::Wallets,
            },
            link(GemListRowTitle::Security, GemListRowIcon::Security),
        ],
        [
            notifications_available.then(|| link(GemListRowTitle::Notifications, GemListRowIcon::Notifications)),
            Some(link(GemListRowTitle::Preferences, GemListRowIcon::Preferences)),
        ]
        .into_iter()
        .flatten()
        .collect(),
        match wallet_connect_available {
            true => vec![link(GemListRowTitle::WalletConnect, GemListRowIcon::WalletConnect)],
            false => vec![],
        },
        [
            Some(link(GemListRowTitle::Support, GemListRowIcon::Support)),
            shows_rewards.then(|| link(GemListRowTitle::Rewards, GemListRowIcon::Rewards)),
            Some(link(GemListRowTitle::AboutUs, GemListRowIcon::AboutUs)),
            developer_enabled.then(|| link(GemListRowTitle::Developer, GemListRowIcon::Developer)),
        ]
        .into_iter()
        .flatten()
        .collect(),
    ]
    .into_iter()
    .filter(|rows: &Vec<GemListRow>| !rows.is_empty())
    .map(|rows| GemListSection {
        title: GemListSectionTitle::None,
        footer: GemListSectionFooter::None,
        rows,
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notifications_offer_the_push_toggle_and_the_price_alerts_link() {
        let sections = notifications_sections(true);

        assert_eq!(
            sections.iter().map(|section| section.rows.clone()).collect::<Vec<_>>(),
            vec![
                vec![GemListRow::Toggle {
                    title: GemListRowTitle::Notifications,
                    value: None,
                    icon: GemListRowIcon::None,
                    is_on: true,
                }],
                vec![GemListRow::Link {
                    title: GemListRowTitle::PriceAlerts,
                    value: None,
                    icon: GemListRowIcon::PriceAlerts,
                }],
            ]
        );
        assert!(matches!(notifications_sections(false)[0].rows[0], GemListRow::Toggle { is_on: false, .. }));
    }

    fn preferences_input(perpetuals_enabled: bool, language: Option<&str>) -> GemPreferencesInput {
        GemPreferencesInput {
            currency: Currency::GBP,
            language: language.map(str::to_string),
            appearance: "System".to_string(),
            perpetuals_enabled,
            perpetual_defaults: GemPerpetualDefaults {
                leverage: 5,
                take_profit_percent: 25,
                stop_loss_percent: 0,
            },
        }
    }

    #[test]
    fn test_the_perpetual_defaults_show_only_once_perpetuals_are_on() {
        assert_eq!(
            preferences_sections(preferences_input(false, Some("English"))).last().map(|section| section.rows.clone()),
            Some(vec![GemListRow::Toggle {
                title: GemListRowTitle::Perpetuals,
                value: None,
                icon: GemListRowIcon::Perpetuals,
                is_on: false,
            }])
        );
        assert_eq!(
            preferences_sections(preferences_input(true, Some("English")))
                .last()
                .map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()),
            Some(vec![GemListRowTitle::Perpetuals, GemListRowTitle::PerpetualLeverage, GemListRowTitle::PerpetualTakeProfit, GemListRowTitle::PerpetualStopLoss])
        );
    }

    #[test]
    fn test_the_perpetual_pickers_name_every_option_and_the_rows_show_the_saved_one() {
        let pickers = perpetual_pickers();

        assert_eq!(pickers.leverage.first().map(|option| option.value), Some(1));
        assert!(
            pickers
                .leverage
                .iter()
                .all(|option| matches!(option.label, GemLocalizedText::Number { ref number } if number.unit == crate::formatted_number::GemNumberUnit::Multiplier))
        );
        assert_eq!(pickers.take_profit.first().map(|option| option.label.clone()), Some(GemLocalizedText::None), "no take profit reads as none");
        assert_eq!(
            preferences_sections(preferences_input(true, None)).last().map(|section| section.rows.clone()).unwrap_or_default()[3],
            GemListRow::Picker {
                title: GemListRowTitle::PerpetualStopLoss,
                value: GemLocalizedText::None,
                icon: GemListRowIcon::None,
            },
            "the saved value reads through the same option label"
        );
    }

    #[test]
    fn test_the_preference_rows_carry_the_flagged_currency_and_drop_the_language_a_platform_cannot_set() {
        let sections = preferences_sections(preferences_input(false, Some("English")));

        assert_eq!(
            sections.first().and_then(|section| section.rows.first()).cloned(),
            Some(GemListRow::Link {
                title: GemListRowTitle::Currency,
                value: Some("\u{1f1ec}\u{1f1e7} GBP".to_string()),
                icon: GemListRowIcon::Currency,
            })
        );
        assert_eq!(
            sections.first().map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()),
            Some(vec![GemListRowTitle::Currency, GemListRowTitle::Language, GemListRowTitle::Appearance, GemListRowTitle::Networks, GemListRowTitle::Contacts])
        );
        assert_eq!(
            preferences_sections(preferences_input(false, None)).first().map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()),
            Some(vec![GemListRowTitle::Currency, GemListRowTitle::Appearance, GemListRowTitle::Networks, GemListRowTitle::Contacts])
        );
    }

    #[test]
    fn test_the_lock_rows_show_only_once_authentication_is_on() {
        let input = |authentication_enabled: bool| GemSecurityInput {
            authentication_enabled,
            authentication_name: Some("Face ID".to_string()),
            lock_period: "Immediately".to_string(),
            privacy_lock_enabled: true,
            privacy_lock_supported: true,
            hide_balance_enabled: false,
        };

        assert_eq!(
            security_sections(input(false)).first().map(|section| section.rows.len()),
            Some(1),
            "a device without authentication offers only the switch that turns it on"
        );
        assert_eq!(
            security_sections(input(true)).first().map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()),
            Some(vec![GemListRowTitle::Authentication, GemListRowTitle::LockPeriod, GemListRowTitle::PrivacyLock])
        );
        assert_eq!(
            security_sections(GemSecurityInput {
                privacy_lock_supported: false,
                ..input(true)
            })
            .first()
            .map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()),
            Some(vec![GemListRowTitle::Authentication, GemListRowTitle::LockPeriod]),
            "a platform without a privacy lock drops that row"
        );
        assert_eq!(
            security_sections(input(true)).last().map(|section| section.rows.clone()),
            Some(vec![GemListRow::Toggle {
                title: GemListRowTitle::HideBalance,
                value: None,
                icon: GemListRowIcon::None,
                is_on: false
            }]),
            "hiding the balance is its own choice, not part of the lock"
        );
    }

    #[test]
    fn test_the_about_screen_offers_the_update_only_when_a_release_is_newer() {
        let plain = about_sections("1.2.3".to_string(), "345".to_string(), None);
        assert_eq!(
            plain.last().map(|section| section.rows.clone()),
            Some(vec![GemListRow::Text {
                title: GemListRowTitle::Version,
                value: "1.2.3 (345)".to_string()
            }]),
            "the row composes the version and the build here, so both apps read the same text"
        );

        let update = about_sections("1.2.3".to_string(), "345".to_string(), Some(Release::new(PlatformStore::AppStore, "1.3.0".to_string(), false)));
        assert_eq!(
            update.last().and_then(|section| section.rows.last().cloned()),
            Some(GemListRow::Url {
                title: GemListRowTitle::UpdateApp,
                value: Some("1.3.0".to_string()),
                icon: GemListRowIcon::AppLogo,
                url: PublicUrl::AppStore.url(),
                target: GemUrlTarget::External,
            }),
            "the row points at the store the release came from, and a store page opens outside the app"
        );
    }

    #[test]
    fn test_the_settings_rows_follow_what_the_device_and_wallet_offer() {
        let titles = |sections: Vec<GemListSection>| sections.iter().map(|section| section.rows.iter().filter_map(row_title).collect::<Vec<_>>()).collect::<Vec<_>>();

        assert_eq!(
            titles(sections(3, true, true, true, true)),
            vec![
                vec![GemListRowTitle::Wallets, GemListRowTitle::Security],
                vec![GemListRowTitle::Notifications, GemListRowTitle::Preferences],
                vec![GemListRowTitle::WalletConnect],
                vec![GemListRowTitle::Support, GemListRowTitle::Rewards, GemListRowTitle::AboutUs, GemListRowTitle::Developer],
            ]
        );

        assert_eq!(
            titles(sections(1, false, false, false, false)),
            vec![
                vec![GemListRowTitle::Wallets, GemListRowTitle::Security],
                vec![GemListRowTitle::Preferences],
                vec![GemListRowTitle::Support, GemListRowTitle::AboutUs],
            ],
            "a device without notifications or WalletConnect drops those rows and their empty section"
        );

        assert_eq!(
            sections(3, false, false, false, false).first().and_then(|section| section.rows.first().cloned()),
            Some(GemListRow::Link {
                title: GemListRowTitle::Wallets,
                value: Some("3".to_string()),
                icon: GemListRowIcon::Wallets,
            }),
            "the wallets row counts the wallets the screen was given"
        );
    }

    fn row_title(row: &GemListRow) -> Option<GemListRowTitle> {
        match row {
            GemListRow::Latency { title, .. }
            | GemListRow::Notice { title, .. }
            | GemListRow::Link { title, .. }
            | GemListRow::Text { title, .. }
            | GemListRow::Amount { title, .. }
            | GemListRow::Rate { title, .. }
            | GemListRow::Quote { title, .. }
            | GemListRow::Ranked { title, .. }
            | GemListRow::AllTime { title, .. }
            | GemListRow::Identifier { title, .. }
            | GemListRow::Duration { title, .. }
            | GemListRow::Label { title, .. }
            | GemListRow::Date { title, .. }
            | GemListRow::Network { title, .. }
            | GemListRow::Url { title, .. }
            | GemListRow::Toggle { title, .. }
            | GemListRow::Picker { title, .. }
            | GemListRow::Lines { title, .. }
            | GemListRow::Provider { title, .. } => Some(*title),
            GemListRow::App { .. }
            | GemListRow::Wallet { .. }
            | GemListRow::Memo { .. }
            | GemListRow::Social { .. }
            | GemListRow::Icon { .. }
            | GemListRow::Address { .. }
            | GemListRow::Explorer { .. }
            | GemListRow::Loading
            | GemListRow::Error { .. } => None,
        }
    }
}
