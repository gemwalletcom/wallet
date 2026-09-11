use primitives::Currency;
use strum::IntoEnumIterator;

use super::model::{GemCurrencies, GemCurrencyRow};
use crate::services::collections::unique;

const DEFAULT_CURRENCIES: [Currency; 7] = [Currency::USD, Currency::EUR, Currency::GBP, Currency::CNY, Currency::JPY, Currency::INR, Currency::RUB];

pub fn currencies(current: Currency, locale: Option<Currency>) -> GemCurrencies {
    let recommended = recommended_currencies(current.clone(), locale);
    let other = other_currencies(&recommended);
    GemCurrencies {
        selected: row(current),
        recommended: recommended.into_iter().map(row).collect(),
        other: other.into_iter().map(row).collect(),
    }
}

fn row(currency: Currency) -> GemCurrencyRow {
    GemCurrencyRow {
        flag: currency.flag().to_string(),
        currency,
    }
}

fn recommended_currencies(current: Currency, locale: Option<Currency>) -> Vec<Currency> {
    unique([current].into_iter().chain(locale).chain(DEFAULT_CURRENCIES))
}

fn other_currencies(recommended: &[Currency]) -> Vec<Currency> {
    Currency::iter().filter(|currency| !recommended.contains(currency)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_currencies_flag_every_row_and_name_the_selected_one() {
        let currencies = currencies(Currency::GBP, Some(Currency::EUR));

        assert_eq!(currencies.selected, row(Currency::GBP));
        assert_eq!(currencies.selected.flag, "🇬🇧");
        assert_eq!(currencies.recommended[1].flag, "🇪🇺");
        assert_eq!(currencies.recommended.len() + currencies.other.len(), Currency::iter().count());
        assert!(currencies.recommended.iter().chain(&currencies.other).all(|row| !row.flag.is_empty()));
    }

    #[test]
    fn test_recommended_currencies_lead_with_the_current_and_locale_currency_once() {
        assert_eq!(
            recommended_currencies(Currency::CHF, Some(Currency::EUR)),
            vec![
                Currency::CHF,
                Currency::EUR,
                Currency::USD,
                Currency::GBP,
                Currency::CNY,
                Currency::JPY,
                Currency::INR,
                Currency::RUB
            ]
        );
        assert_eq!(recommended_currencies(Currency::USD, None), DEFAULT_CURRENCIES.to_vec());
        assert_eq!(recommended_currencies(Currency::USD, Some(Currency::USD)).len(), DEFAULT_CURRENCIES.len());
    }

    #[test]
    fn test_other_currencies_exclude_the_recommended_ones() {
        let recommended = recommended_currencies(Currency::CHF, None);
        let others = other_currencies(&recommended);

        assert!(others.iter().all(|currency| !recommended.contains(currency)));
        assert!(others.contains(&Currency::AUD));
        assert_eq!(others.len() + recommended.len(), Currency::iter().count());
    }
}
