use num_bigint::BigUint;
use primitives::{FiatQuote, FiatQuoteType};
use rand::RngExt;

use super::model::GemFiatAmountCheck;
use crate::config::fiat_config::FiatConfig;

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
    match (quote_type, quote) {
        (FiatQuoteType::Sell, Some(quote)) if quote.value > *available => GemFiatAmountCheck::InsufficientBalance {
            required: quote.value.clone(),
            available: available.clone(),
        },
        _ => GemFiatAmountCheck::Valid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::fiat_config::get_fiat_config;
    use primitives::{Asset, Chain, FiatProvider, FiatProviderName};

    fn quote(value: u32) -> FiatQuote {
        FiatQuote::new(
            "quote".to_string(),
            Asset::from_chain(Chain::Ethereum),
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
            1.0,
            BigUint::from(value),
            10,
            vec![],
        )
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
                required: BigUint::from(200u32),
                available: BigUint::from(100u32),
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
    fn test_default_and_random_amounts_follow_the_config() {
        let config = get_fiat_config();
        assert_eq!(default_amount(&config, FiatQuoteType::Buy), 50);
        assert_eq!(default_amount(&config, FiatQuoteType::Sell), 100);
        let random = random_amount(&config);
        assert!((50..1000).contains(&random));
    }
}
