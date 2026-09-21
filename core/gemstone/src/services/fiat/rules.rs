use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Currency, FiatProviderName, FiatQuote, FiatQuoteType, FiatTransactionAssetData, FiatTransactionStatus};
use rand::RngExt;

use super::model::{GemFiatAmountCheck, GemFiatQuoteRow, GemFiatTransactionBadge, GemFiatTransactionRow, GemFiatTransactionStatus};
use crate::config::fiat_config::FiatConfig;
use crate::formatted_number::GemFormattedNumber;
use crate::precision::{GemCurrencyStyle, GemValueStyle};
use crate::services::assets::GemAssetAction;
use crate::services::swap::GemAssetRate;

pub fn default_amount(config: &FiatConfig, quote_type: FiatQuoteType) -> u32 {
    match quote_type {
        FiatQuoteType::Buy => config.default_buy_amount as u32,
        FiatQuoteType::Sell => config.default_sell_amount as u32,
    }
}

pub fn random_amount(config: &FiatConfig) -> u32 {
    rand::rng().random_range(config.default_buy_amount as u32..config.random_max_amount as u32)
}

pub fn amount_check(config: &FiatConfig, quote_type: FiatQuoteType, amount: f64, quote: Option<&FiatQuote>, available: &BigUint, currency: Currency) -> GemFiatAmountCheck {
    if amount < config.minimum_amount as f64 {
        return GemFiatAmountCheck::BelowMinimum {
            minimum: GemFormattedNumber::currency(config.minimum_amount as f64, currency, GemCurrencyStyle::Currency),
        };
    }
    if amount > config.maximum_amount as f64 {
        return GemFiatAmountCheck::AboveMaximum {
            maximum: GemFormattedNumber::currency(config.maximum_amount as f64, currency, GemCurrencyStyle::Currency),
        };
    }
    match (quote_type, quote) {
        (FiatQuoteType::Sell, Some(quote)) if quote_value(quote).is_some_and(|value| value > *available) => GemFiatAmountCheck::InsufficientBalance { title: quote.asset.display_title() },
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
        Ok(value) if value > 0.0 && value.fract() == 0.0 => FiatAmountInput::Value(value),
        Ok(value) if value <= 0.0 => FiatAmountInput::Empty,
        Ok(_) | Err(_) => FiatAmountInput::Invalid,
    }
}

pub fn selected_quote(quotes: &[FiatQuote], preferred: Option<FiatProviderName>) -> Option<FiatQuote> {
    quotes.iter().find(|quote| preferred.is_some_and(|provider| quote.provider.id == provider)).or_else(|| quotes.first()).cloned()
}

pub fn quote_row(quote: &FiatQuote, asset_price: Option<f64>) -> GemFiatQuoteRow {
    let fiat_amount = match (quote.quote_type, asset_price) {
        (FiatQuoteType::Buy, Some(price)) if price > 0.0 => price * quote.crypto_amount,
        _ => quote.fiat_amount,
    };
    GemFiatQuoteRow {
        quote_id: quote.id.clone(),
        provider: quote.provider.id,
        provider_name: quote.provider.name.clone(),
        crypto_amount: GemFormattedNumber::amount(quote.crypto_amount, Some(quote.asset.symbol.clone()), GemValueStyle::Auto),
        fiat_amount: GemFormattedNumber::currency_code(fiat_amount, quote.fiat_currency.clone(), GemCurrencyStyle::Fiat),
        rate: (quote.crypto_amount > 0.0).then(|| GemAssetRate {
            base_symbol: quote.asset.symbol.clone(),
            value: GemFormattedNumber::currency_code(quote.fiat_amount / quote.crypto_amount, quote.fiat_currency.clone(), GemCurrencyStyle::Currency),
        }),
    }
}

pub fn quote_action(quote_type: &FiatQuoteType) -> GemAssetAction {
    match quote_type {
        FiatQuoteType::Buy => GemAssetAction::Buy,
        FiatQuoteType::Sell => GemAssetAction::Sell,
    }
}

pub fn transaction_status(status: FiatTransactionStatus) -> GemFiatTransactionStatus {
    match status {
        FiatTransactionStatus::Complete => GemFiatTransactionStatus { badge: None, is_dimmed: false },
        FiatTransactionStatus::Pending => GemFiatTransactionStatus {
            badge: Some(GemFiatTransactionBadge::Pending),
            is_dimmed: false,
        },
        FiatTransactionStatus::Failed => GemFiatTransactionStatus {
            badge: Some(GemFiatTransactionBadge::Failed),
            is_dimmed: true,
        },
        FiatTransactionStatus::Unknown => GemFiatTransactionStatus { badge: None, is_dimmed: true },
    }
}

pub fn transaction_row(data: &FiatTransactionAssetData) -> GemFiatTransactionRow {
    let status = transaction_status(data.status.clone());
    GemFiatTransactionRow {
        quote_type: data.transaction_type,
        provider: data.provider,
        subtitle: format!("{} ({})", data.asset.name, data.provider.name()),
        value: GemFormattedNumber::amount(BigNumberFormatter::f64_value(data.value.to_string(), data.asset.decimals as u32), Some(data.asset.symbol.clone()), GemValueStyle::Short),
        fiat_value: GemFormattedNumber::currency_code(data.fiat_amount, data.fiat_currency.clone(), GemCurrencyStyle::Fiat),
        badge: status.badge,
        is_dimmed: status.is_dimmed,
        details_url: data.details_url.clone(),
    }
}

pub fn quote_value(quote: &FiatQuote) -> Option<BigUint> {
    let amount = format!("{:.precision$}", quote.crypto_amount, precision = quote.asset.decimals as usize);
    BigNumberFormatter::value_from_amount_biguint(&amount, quote.asset.decimals as u32).ok()
}

#[cfg(test)]
mod tests {
    use crate::formatted_number::GemNumberUnit;

    #[test]
    fn test_a_fiat_transaction_row_badges_pending_and_failed_and_dims_what_did_not_complete() {
        let status = transaction_status;
        assert_eq!(status(FiatTransactionStatus::Complete), GemFiatTransactionStatus { badge: None, is_dimmed: false });
        assert_eq!(
            status(FiatTransactionStatus::Pending),
            GemFiatTransactionStatus {
                badge: Some(GemFiatTransactionBadge::Pending),
                is_dimmed: false
            }
        );
        assert_eq!(
            status(FiatTransactionStatus::Failed),
            GemFiatTransactionStatus {
                badge: Some(GemFiatTransactionBadge::Failed),
                is_dimmed: true
            }
        );
        assert_eq!(status(FiatTransactionStatus::Unknown), GemFiatTransactionStatus { badge: None, is_dimmed: true });
    }

    use super::*;
    use crate::config::fiat_config::get_fiat_config;
    use primitives::{Asset, Chain};

    #[test]
    fn test_a_transaction_row_names_its_asset_provider_value_and_status() {
        let data = FiatTransactionAssetData {
            id: "1".to_string(),
            asset: Asset::from_chain(Chain::Ethereum),
            transaction_type: FiatQuoteType::Buy,
            provider: FiatProviderName::MoonPay,
            status: FiatTransactionStatus::Pending,
            fiat_amount: 25.0,
            fiat_currency: "USD".to_string(),
            value: BigUint::from(1_500_000_000_000_000_000u64),
            created_at: chrono::Utc::now(),
            details_url: Some("https://moonpay.test/1".to_string()),
        };

        let row = transaction_row(&data);

        assert_eq!(row.subtitle, "Ethereum (MoonPay)");
        assert_eq!(row.value.value, 1.5);
        assert_eq!(row.value.unit, GemNumberUnit::Symbol { symbol: "ETH".to_string() });
        assert_eq!(row.fiat_value.value, 25.0);
        assert_eq!(row.badge, Some(GemFiatTransactionBadge::Pending));
        assert!(!row.is_dimmed);
        assert_eq!(row.details_url.as_deref(), Some("https://moonpay.test/1"));
    }

    #[test]
    fn test_row_prices_a_buy_off_the_asset_price_and_a_sell_off_the_quote() {
        let buy = FiatQuote {
            crypto_amount: 2.0,
            ..FiatQuote::mock(FiatProviderName::Transak)
        };

        assert_eq!(quote_row(&buy, Some(30.0)).fiat_amount.value, 60.0);
        assert_eq!(quote_row(&buy, Some(0.0)).fiat_amount.value, 100.0);
        assert_eq!(quote_row(&buy, None).fiat_amount.value, 100.0);

        let mut sell = buy.clone();
        sell.quote_type = FiatQuoteType::Sell;
        assert_eq!(quote_row(&sell, Some(30.0)).fiat_amount.value, 100.0);

        let row = quote_row(&buy, Some(30.0));
        assert_eq!(row.fiat_amount.unit, GemNumberUnit::Currency { code: buy.fiat_currency.clone() });
        assert_eq!(row.crypto_amount.unit, GemNumberUnit::Symbol { symbol: buy.asset.symbol.clone() });
    }

    #[test]
    fn test_row_names_the_rate_it_prices_and_has_none_when_the_quote_buys_nothing() {
        let mut quote = FiatQuote {
            crypto_amount: 4.0,
            ..FiatQuote::mock(FiatProviderName::Transak)
        };
        assert_eq!(
            quote_row(&quote, None).rate,
            Some(GemAssetRate {
                base_symbol: quote.asset.symbol.clone(),
                value: GemFormattedNumber::currency(25.0, Currency::USD, GemCurrencyStyle::Currency)
            })
        );

        quote.crypto_amount = 0.0;
        assert_eq!(quote_row(&quote, None).rate, None);
    }

    #[test]
    fn test_quote_value_is_derived_from_the_amount_and_the_asset_precision() {
        let quote = FiatQuote {
            asset: Asset::from_chain(Chain::Ethereum),
            crypto_amount: 200.0 / 10f64.powi(18),
            ..FiatQuote::mock(FiatProviderName::Transak)
        };
        assert_eq!(quote_value(&quote), Some(BigUint::from(200u32)));
        let whole = FiatQuote { crypto_amount: 1.5, ..quote };
        assert_eq!(quote_value(&whole), Some(BigUint::from(1_500_000_000_000_000_000u64)));
    }

    #[test]
    fn test_amount_check_orders_range_before_balance() {
        let config = get_fiat_config();
        let two_hundred = FiatQuote {
            asset: Asset::from_chain(Chain::Ethereum),
            quote_type: FiatQuoteType::Sell,
            crypto_amount: 200.0 / 10f64.powi(18),
            ..FiatQuote::mock(FiatProviderName::Transak)
        };
        let hundred = FiatQuote {
            crypto_amount: 100.0 / 10f64.powi(18),
            ..two_hundred.clone()
        };
        assert_eq!(
            amount_check(&config, FiatQuoteType::Buy, 4.99, None, &BigUint::ZERO, Currency::USD),
            GemFiatAmountCheck::BelowMinimum {
                minimum: GemFormattedNumber::currency(5.0, Currency::USD, GemCurrencyStyle::Currency)
            }
        );
        assert_eq!(
            amount_check(&config, FiatQuoteType::Sell, 10_001.0, Some(&hundred), &BigUint::ZERO, Currency::USD),
            GemFiatAmountCheck::AboveMaximum {
                maximum: GemFormattedNumber::currency(10_000.0, Currency::USD, GemCurrencyStyle::Currency)
            }
        );
        assert_eq!(
            amount_check(&config, FiatQuoteType::Sell, 100.0, Some(&two_hundred), &BigUint::from(100u32), Currency::USD),
            GemFiatAmountCheck::InsufficientBalance {
                title: Asset::from_chain(Chain::Ethereum).display_title()
            }
        );
        assert_eq!(amount_check(&config, FiatQuoteType::Sell, 100.0, Some(&hundred), &BigUint::from(100u32), Currency::USD), GemFiatAmountCheck::Valid);
        assert_eq!(amount_check(&config, FiatQuoteType::Sell, 100.0, None, &BigUint::ZERO, Currency::USD), GemFiatAmountCheck::Valid);
        assert_eq!(amount_check(&config, FiatQuoteType::Buy, 100.0, Some(&two_hundred), &BigUint::ZERO, Currency::USD), GemFiatAmountCheck::Valid);
    }

    #[test]
    fn test_parse_amount_takes_whole_amounts_only_and_treats_zero_as_empty() {
        assert!(matches!(parse_amount("1 000"), FiatAmountInput::Value(value) if value == 1000.0));
        assert!(matches!(parse_amount(" 12 "), FiatAmountInput::Value(value) if value == 12.0));
        assert!(matches!(parse_amount(""), FiatAmountInput::Empty));
        assert!(matches!(parse_amount("0"), FiatAmountInput::Empty));
        assert!(matches!(parse_amount(" 12,5 "), FiatAmountInput::Invalid));
        assert!(matches!(parse_amount("12.5"), FiatAmountInput::Invalid));
        assert!(matches!(parse_amount("abc"), FiatAmountInput::Invalid));
    }

    #[test]
    fn test_selected_quote_prefers_the_chosen_provider_and_falls_back_to_the_first() {
        let quotes = vec![
            FiatQuote {
                value: BigUint::from(1u32),
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
            FiatQuote {
                value: BigUint::from(2u32),
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
        ];
        assert_eq!(selected_quote(&quotes, None).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&quotes, Some(FiatProviderName::Transak)).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&quotes, Some(FiatProviderName::Banxa)).map(|quote| quote.value), Some(BigUint::from(1u32)));
        assert_eq!(selected_quote(&[], Some(FiatProviderName::Transak)), None);
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
