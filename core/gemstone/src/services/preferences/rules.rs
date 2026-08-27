use std::str::FromStr;

use primitives::currency::Currency;

pub fn default_currency(locale_currency: Option<String>) -> Currency {
    locale_currency.and_then(|code| Currency::from_str(&code).ok()).unwrap_or(Currency::USD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_currency() {
        assert_eq!(default_currency(Some("EUR".to_string())), Currency::EUR);
        assert_eq!(default_currency(Some("XXX".to_string())), Currency::USD);
        assert_eq!(default_currency(None), Currency::USD);
    }
}
