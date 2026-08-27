use std::str::FromStr;

use primitives::currency::Currency;

const REVIEW_REQUEST_LAUNCHES: u32 = 5;

pub fn should_request_review(launches_count: u32, rate_application_shown: bool) -> bool {
    launches_count >= REVIEW_REQUEST_LAUNCHES && !rate_application_shown
}

pub fn default_currency(locale_currency: Option<String>) -> Currency {
    locale_currency.and_then(|code| Currency::from_str(&code).ok()).unwrap_or(Currency::USD)
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
    fn test_default_currency() {
        assert_eq!(default_currency(Some("EUR".to_string())), Currency::EUR);
        assert_eq!(default_currency(Some("XXX".to_string())), Currency::USD);
        assert_eq!(default_currency(None), Currency::USD);
    }
}
