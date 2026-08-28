use std::str::FromStr;

use primitives::Appearance;

use primitives::currency::Currency;

const REVIEW_REQUEST_LAUNCHES: u32 = 5;
const ASK_NOTIFICATIONS_COOLDOWN_SECONDS: u64 = 30 * 24 * 60 * 60;

pub fn should_request_review(launches_count: u32, rate_application_shown: bool) -> bool {
    launches_count >= REVIEW_REQUEST_LAUNCHES && !rate_application_shown
}

pub fn should_ask_notifications(last_asked_at: u64, now: u64) -> bool {
    now.saturating_sub(last_asked_at) >= ASK_NOTIFICATIONS_COOLDOWN_SECONDS
}

pub fn default_currency(locale_currency: Option<String>) -> Currency {
    locale_currency.and_then(|code| Currency::from_str(&code).ok()).unwrap_or(Currency::USD)
}

pub fn swap_slippage_bps(value: Option<String>) -> Option<u32> {
    value.and_then(|value| value.parse::<u32>().ok()).filter(|bps| *bps > 0)
}

pub fn percent_or_default(value: Option<String>, default: u8) -> u8 {
    value.and_then(|value| value.parse::<u8>().ok()).filter(|value| *value > 0).unwrap_or(default)
}

pub fn appearance(value: Option<String>) -> Appearance {
    match value.as_deref() {
        Some("light") => Appearance::Light,
        Some("dark") => Appearance::Dark,
        _ => Appearance::System,
    }
}

pub fn appearance_value(appearance: Appearance) -> &'static str {
    match appearance {
        Appearance::System => "system",
        Appearance::Light => "light",
        Appearance::Dark => "dark",
    }
}

pub fn flag(value: Option<String>) -> bool {
    value.as_deref() == Some("true")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_requested_after_five_launches_until_shown() {
        assert!(!should_request_review(4, false));
        assert!(should_request_review(5, false));
        assert!(!should_request_review(9, true));
    }

    #[test]
    fn test_asking_for_notifications_again_waits_a_month() {
        let now = 1_700_000_000;

        assert!(should_ask_notifications(0, now));
        assert!(should_ask_notifications(now - ASK_NOTIFICATIONS_COOLDOWN_SECONDS, now));
        assert!(!should_ask_notifications(now - ASK_NOTIFICATIONS_COOLDOWN_SECONDS + 1, now));
        assert!(!should_ask_notifications(now, now));
    }

    #[test]
    fn test_default_currency() {
        assert_eq!(default_currency(Some("EUR".to_string())), Currency::EUR);
        assert_eq!(default_currency(Some("XXX".to_string())), Currency::USD);
        assert_eq!(default_currency(None), Currency::USD);
    }

    #[test]
    fn test_swap_slippage_and_percent_defaults() {
        assert_eq!(swap_slippage_bps(Some("150".to_string())), Some(150));
        assert_eq!(swap_slippage_bps(Some("0".to_string())), None);
        assert_eq!(swap_slippage_bps(None), None);
        assert_eq!(percent_or_default(Some("25".to_string()), 5), 25);
        assert_eq!(percent_or_default(Some("0".to_string()), 5), 5);
        assert_eq!(percent_or_default(Some("x".to_string()), 5), 5);
        assert_eq!(percent_or_default(None, 5), 5);
    }

    #[test]
    fn test_appearance_defaults_to_system() {
        assert_eq!(appearance(Some("dark".to_string())), Appearance::Dark);
        assert_eq!(appearance(Some("nope".to_string())), Appearance::System);
        assert_eq!(appearance(None), Appearance::System);
    }

    #[test]
    fn test_flag_is_only_true_for_true() {
        assert!(flag(Some("true".to_string())));
        assert!(!flag(Some("false".to_string())));
        assert!(!flag(Some("1".to_string())));
        assert!(!flag(None));
    }
}
