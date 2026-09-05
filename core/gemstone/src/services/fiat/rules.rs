use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{FiatQuote, FiatQuoteType};
use rand::RngExt;

use super::model::GemFiatAmountCheck;
use crate::config::fiat_config::FiatConfig;
use crate::services::balance::GemBalanceRequirement;

pub fn default_amount(config: &FiatConfig, quote_type: FiatQuoteType) -> u32 {
    match quote_type {
        FiatQuoteType::Buy => config.default_buy_amount as u32,
        FiatQuoteType::Sell => config.default_sell_amount as u32,
    }
}

pub fn random_amount(config: &FiatConfig) -> u32 {
    rand::rng().random_range(config.default_buy_amount as u32..config.random_max_amount as u32)
}

pub fn amount_check(config: &FiatConfig, quote_type: FiatQuoteType, amount: f64, quote: Option<&FiatQuote>, available: &BigUint) -> GemFiatAmountCheck {
    if amount < config.minimum_amount as f64 {
        return GemFiatAmountCheck::BelowMinimum {
            minimum: config.minimum_amount as u32,
        };
    }
    if amount > config.maximum_amount as f64 {
        return GemFiatAmountCheck::AboveMaximum {
            maximum: config.maximum_amount as u32,
        };
    }
    match (quote_type, quote.and_then(quote_value)) {
        (FiatQuoteType::Sell, Some(value)) if value > *available => GemFiatAmountCheck::InsufficientBalance {
            requirement: GemBalanceRequirement::new(value.into(), available.clone().into()),
        },
        _ => GemFiatAmountCheck::Valid,
    }
}

pub enum FiatAmountInput {
    Empty,
    Invalid,
    Value(f64),
}

pub fn parse_amount(text: &str) -> FiatAmountInput {
    let normalized: String = text.trim().replace(',', ".").chars().filter(|character| !character.is_whitespace()).collect();
    if normalized.is_empty() {
        return FiatAmountInput::Empty;
    }
    match normalized.parse::<f64>() {
        Ok(value) if value > 0.0 => FiatAmountInput::Value(value),
        Ok(_) => FiatAmountInput::Empty,
        Err(_) => FiatAmountInput::Invalid,
    }
}

pub fn selected_quote(quotes: &[FiatQuote], preferred: Option<&str>) -> Option<FiatQuote> {
    quotes
        .iter()
        .find(|quote| preferred.is_some_and(|provider| quote.provider.id.id() == provider))
        .or_else(|| quotes.first())
        .cloned()
}

pub fn quote_value(quote: &FiatQuote) -> Option<BigUint> {
    let amount = format!("{:.precision$}", quote.crypto_amount, precision = quote.asset.decimals as usize);
    BigNumberFormatter::value_from_amount_biguint(&amount, quote.asset.decimals as u32).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::fiat_config::get_fiat_config;
    use num_bigint::BigInt;
    use primitives::{Asset, Chain, FiatProvider, FiatProviderName};

    fn quote(value: u32) -> FiatQuote {
        let asset = Asset::from_chain(Chain::Ethereum);
        let crypto_amount = value as f64 / 10f64.powi(asset.decimals);
        FiatQuote::new(
            "quote".to_string(),
            asset,
            FiatProvider {
                id: FiatProviderName::Transak,
                name: "Provider".to_string(),
                image_url: None,
                priority: None,
                threshold_bps: None,
                enabled: true,
                buy_enabled: true,
                sell_enabled: true,
                payment_methods: vec![],
            },
            FiatQuoteType::Sell,
            100.0,
            "USD".to_string(),
            crypto_amount,
            BigUint::from(value),
            10,
            vec![],
        )
    }

    #[test]
    fn test_quote_value_is_derived_from_the_amount_and_the_asset_precision() {
        assert_eq!(quote_value(&quote(200)), Some(BigUint::from(200u32)));
        let mut whole = quote(1);
        whole.crypto_amount = 1.5;
        assert_eq!(quote_value(&whole), Some(BigUint::from(1_500_000_000_000_000_000u64)));
    }

    #[test]
    fn test_amount_check_orders_range_before_balance() {
        let config = get_fiat_config();
        assert_eq!(
            amount_check(&config, FiatQuoteType::Buy, 4.99, None, &BigUint::ZERO),
            GemFiatAmountCheck::BelowMinimum { minimum: 5 }
        );
        assert_eq!(
            amount_check(&config, FiatQuoteType::Sell, 10_001.0, Some(&quote(1)), &BigUint::ZERO),
            GemFiatAmountCheck::AboveMaximum { maximum: 10_000 }
        );
        assert_eq!(
            amount_check(&config, FiatQuoteType::Sell, 100.0, Some(&quote(200)), &BigUint::from(100u32)),
            GemFiatAmountCheck::InsufficientBalance {
                requirement: GemBalanceRequirement::new(BigInt::from(200), BigInt::from(100))
            }
        );
        assert_eq!(
            amount_check(&config, FiatQuoteType::Sell, 100.0, Some(&quote(100)), &BigUint::from(100u32)),
            GemFiatAmountCheck::Valid
        );
        assert_eq!(amount_check(&config, FiatQuoteType::Sell, 100.0, None, &BigUint::ZERO), GemFiatAmountCheck::Valid);
        assert_eq!(
            amount_check(&config, FiatQuoteType::Buy, 100.0, Some(&quote(200)), &BigUint::ZERO),
            GemFiatAmountCheck::Valid
        );
    }

    #[test]
    fn test_parse_amount_accepts_a_decimal_comma_and_treats_zero_as_empty() {
        assert!(matches!(parse_amount(" 12,5 "), FiatAmountInput::Value(value) if value == 12.5));
        assert!(matches!(parse_amount("1 000"), FiatAmountInput::Value(value) if value == 1000.0));
        assert!(matches!(parse_amount(""), FiatAmountInput::Empty));
        assert!(matches!(parse_amount("0"), FiatAmountInput::Empty));
        assert!(matches!(parse_amount("abc"), FiatAmountInput::Invalid));
    }

    #[test]
    fn test_selected_quote_prefers_the_chosen_provider_and_falls_back_to_the_first() {
        let quotes = vec![quote(1), quote(2)];
        assert_eq!(selected_quote(&quotes, None).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&quotes, Some("transak")).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&quotes, Some("banxa")).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&[], Some("transak")), None);
    }

    #[test]
    fn test_default_and_random_amounts_follow_the_config() {
        let config = get_fiat_config();
        assert_eq!(default_amount(&config, FiatQuoteType::Buy), 50);
        assert_eq!(default_amount(&config, FiatQuoteType::Sell), 100);
        let random = random_amount(&config);
        assert!((50..1000).contains(&random));
    }
}
