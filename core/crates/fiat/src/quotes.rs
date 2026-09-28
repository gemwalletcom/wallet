use std::cmp::Ordering;
use std::collections::HashSet;
use std::error::Error;
use std::time::Instant;

use num_bigint::BigUint;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, FiatProvider as PrimitiveFiatProvider, FiatQuote, FiatQuoteRequest, FiatQuoteType, PaymentType, sort_by_priority_then_amount};

use crate::FiatProvider;
use crate::model::FiatMapping;

pub fn is_provider_eligible(db_provider: &PrimitiveFiatProvider, countries: &HashSet<String>, mapping: Option<&FiatMapping>, country_code: &str, request: &FiatQuoteRequest) -> bool {
    let is_enabled = match request.quote_type {
        FiatQuoteType::Buy => db_provider.is_buy_enabled(),
        FiatQuoteType::Sell => db_provider.is_sell_enabled(),
    };
    if !is_enabled {
        return false;
    }
    let Some(mapping) = mapping else {
        return false;
    };
    if !countries.contains(country_code) || mapping.unsupported_countries.contains_key(country_code) {
        return false;
    }
    let limits = match request.quote_type {
        FiatQuoteType::Buy => &mapping.buy_limits,
        FiatQuoteType::Sell => &mapping.sell_limits,
    };
    limits.iter().any(|limit| {
        limit.currency.as_ref() == request.currency
            && (db_provider.payment_methods.is_empty() || db_provider.payment_methods.contains(&limit.payment_type))
            && limit.min_amount.is_none_or(|minimum| request.amount >= minimum)
            && limit.max_amount.is_none_or(|maximum| request.amount <= maximum)
    })
}

pub fn compare_quotes(quote_type: FiatQuoteType, a: &FiatQuote, b: &FiatQuote, providers: &[PrimitiveFiatProvider]) -> Ordering {
    let ascending = match quote_type {
        FiatQuoteType::Buy => false,
        FiatQuoteType::Sell => true,
    };
    sort_by_priority_then_amount(a.provider.id.as_ref(), b.provider.id.as_ref(), &a.crypto_amount, &b.crypto_amount, providers, ascending)
}

pub async fn get_provider_quote(provider: &(dyn FiatProvider + Send + Sync), request: &FiatQuoteRequest, asset: &Asset, mapping: &FiatMapping, db_payment_methods: &[PaymentType]) -> Result<FiatQuote, Box<dyn Error + Send + Sync>> {
    let start = Instant::now();
    let response = match request.quote_type {
        FiatQuoteType::Buy => provider.get_quote_buy(request.clone(), mapping.clone()).await,
        FiatQuoteType::Sell => provider.get_quote_sell(request.clone(), mapping.clone()).await,
    }?;

    if response.fiat_amount <= 0.0 || response.crypto_amount <= 0.0 {
        return Err("Invalid quote amounts".into());
    }

    let latency = start.elapsed().as_millis() as u64;
    let payment_methods = if !response.payment_methods.is_empty() {
        response.payment_methods
    } else if !db_payment_methods.is_empty() {
        db_payment_methods.to_vec()
    } else {
        provider.payment_methods().await
    };
    let value = quote_value(asset, response.crypto_amount)?;
    Ok(FiatQuote::new(
        response.quote_id,
        asset.clone(),
        provider.name().as_fiat_provider(),
        request.quote_type,
        response.fiat_amount,
        request.currency.clone(),
        response.crypto_amount,
        value,
        latency,
        payment_methods,
    ))
}

fn quote_value(asset: &Asset, crypto_amount: f64) -> Result<BigUint, Box<dyn Error + Send + Sync>> {
    let amount = format!("{crypto_amount:.precision$}", precision = asset.decimals as usize);
    Ok(BigNumberFormatter::value_from_amount_biguint(&amount, asset.decimals as u32)?)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use primitives::{Chain, FiatAssetSymbol, FiatProviderName, fiat_assets::FiatAssetLimits};

    use super::*;

    fn sorted(quote_type: FiatQuoteType, mut quotes: Vec<FiatQuote>, providers: &[PrimitiveFiatProvider]) -> Vec<FiatProviderName> {
        quotes.sort_by(|a, b| compare_quotes(quote_type, a, b, providers));
        quotes.into_iter().map(|quote| quote.provider.id).collect()
    }

    #[test]
    fn test_quote_value() {
        assert_eq!(quote_value(&Asset::from_chain(Chain::Ethereum), 0.000000000000000001_f64).unwrap(), BigUint::from(1u64));
    }

    #[test]
    fn test_is_provider_eligible() {
        let countries = HashSet::from(["US".to_string()]);
        let provider = PrimitiveFiatProvider {
            payment_methods: vec![PaymentType::Card],
            ..PrimitiveFiatProvider::mock(FiatProviderName::Banxa)
        };
        let mapping = FiatMapping {
            asset: Asset::from_chain(Chain::Tron),
            asset_symbol: FiatAssetSymbol {
                symbol: "TRX".to_string(),
                network: Some("TRON".to_string()),
            },
            unsupported_countries: HashMap::new(),
            buy_limits: vec![FiatAssetLimits {
                min_amount: Some(10.0),
                max_amount: Some(15_000.0),
                ..FiatAssetLimits::mock()
            }],
            sell_limits: vec![],
        };
        let mut request = FiatQuoteRequest::mock();

        request.amount = 5.0;
        assert!(!is_provider_eligible(&provider, &countries, Some(&mapping), "US", &request));
        request.amount = 10.0;
        assert!(is_provider_eligible(&provider, &countries, Some(&mapping), "US", &request));
        request.amount = 15_000.0;
        assert!(is_provider_eligible(&provider, &countries, Some(&mapping), "US", &request));
        request.amount = 15_001.0;
        assert!(!is_provider_eligible(&provider, &countries, Some(&mapping), "US", &request));
        request.amount = 100.0;
        assert!(!is_provider_eligible(&provider, &countries, Some(&mapping), "DE", &request));
        assert!(!is_provider_eligible(&provider, &countries, None, "US", &request));
        request.currency = "EUR".to_string();
        assert!(!is_provider_eligible(&provider, &countries, Some(&mapping), "US", &request));
    }

    #[test]
    fn test_compare_quotes() {
        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, None),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, None),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Transak, 3, None),
        ];
        let quotes = vec![
            FiatQuote {
                crypto_amount: 0.50,
                ..FiatQuote::mock(FiatProviderName::Paybis)
            },
            FiatQuote {
                crypto_amount: 0.45,
                ..FiatQuote::mock(FiatProviderName::MoonPay)
            },
            FiatQuote {
                crypto_amount: 0.40,
                ..FiatQuote::mock(FiatProviderName::Flashnet)
            },
            FiatQuote {
                crypto_amount: 0.47,
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
            FiatQuote {
                crypto_amount: 0.48,
                ..FiatQuote::mock(FiatProviderName::Mercuryo)
            },
        ];
        assert_eq!(
            sorted(FiatQuoteType::Buy, quotes, &providers),
            vec![FiatProviderName::MoonPay, FiatProviderName::Mercuryo, FiatProviderName::Transak, FiatProviderName::Paybis, FiatProviderName::Flashnet]
        );

        let providers = vec![
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(1000)),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Mercuryo, 2, Some(500)),
            PrimitiveFiatProvider::mock_with_priority(FiatProviderName::Transak, 3, None),
        ];
        let quotes = vec![
            FiatQuote {
                crypto_amount: 0.52,
                ..FiatQuote::mock(FiatProviderName::Paybis)
            },
            FiatQuote {
                crypto_amount: 0.60,
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
            FiatQuote {
                crypto_amount: 0.48,
                ..FiatQuote::mock(FiatProviderName::Mercuryo)
            },
            FiatQuote {
                crypto_amount: 0.45,
                ..FiatQuote::mock(FiatProviderName::MoonPay)
            },
        ];
        assert_eq!(
            sorted(FiatQuoteType::Buy, quotes, &providers),
            vec![FiatProviderName::Transak, FiatProviderName::MoonPay, FiatProviderName::Mercuryo, FiatProviderName::Paybis]
        );

        let providers = vec![PrimitiveFiatProvider::mock_with_priority(FiatProviderName::MoonPay, 1, Some(100))];
        let quotes = vec![
            FiatQuote {
                crypto_amount: 0.0773,
                ..FiatQuote::mock(FiatProviderName::MoonPay)
            },
            FiatQuote {
                crypto_amount: 0.0759,
                ..FiatQuote::mock(FiatProviderName::Mercuryo)
            },
            FiatQuote {
                crypto_amount: 0.07505,
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
            FiatQuote {
                crypto_amount: 0.07721,
                ..FiatQuote::mock(FiatProviderName::Paybis)
            },
        ];
        assert_eq!(
            sorted(FiatQuoteType::Buy, quotes, &providers),
            vec![FiatProviderName::MoonPay, FiatProviderName::Paybis, FiatProviderName::Mercuryo, FiatProviderName::Transak]
        );

        let quotes = vec![
            FiatQuote {
                crypto_amount: 0.036108,
                ..FiatQuote::mock(FiatProviderName::MoonPay)
            },
            FiatQuote {
                crypto_amount: 0.03311059,
                ..FiatQuote::mock(FiatProviderName::Mercuryo)
            },
            FiatQuote {
                crypto_amount: 0.03086637,
                ..FiatQuote::mock(FiatProviderName::Transak)
            },
        ];
        assert_eq!(sorted(FiatQuoteType::Sell, quotes, &[]), vec![FiatProviderName::Transak, FiatProviderName::Mercuryo, FiatProviderName::MoonPay]);
    }
}
