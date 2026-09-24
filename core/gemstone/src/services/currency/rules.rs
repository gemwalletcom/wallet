use std::collections::HashMap;

use primitives::Currency;
use strum::IntoEnumIterator;

use super::model::{GemCurrencyRow, GemCurrencySection, GemCurrencySectionKind};
use crate::services::collections::unique;

const DEFAULT_CURRENCIES: [Currency; 7] = [Currency::USD, Currency::EUR, Currency::GBP, Currency::CNY, Currency::JPY, Currency::INR, Currency::RUB];

pub fn sections(current: Currency, locale: Option<Currency>, query: &str, localized_names: &HashMap<String, String>, rated: &[Currency]) -> Vec<GemCurrencySection> {
    let offered = |currency: &Currency| rated.is_empty() || *currency == current || *currency == Currency::USD || rated.contains(currency);
    let recommended: Vec<Currency> = recommended_currencies(current.clone(), locale).into_iter().filter(offered).collect();
    let other: Vec<Currency> = other_currencies(&recommended).into_iter().filter(offered).collect();
    let query = query.trim().to_lowercase();
    [(GemCurrencySectionKind::Recommended, recommended), (GemCurrencySectionKind::All, other)]
        .into_iter()
        .filter_map(|(kind, currencies)| {
            let rows = currencies
                .into_iter()
                .filter_map(|currency| {
                    let name = localized_names.get(currency.as_ref()).map(String::as_str).unwrap_or("");
                    if !currency.as_ref().to_lowercase().contains(&query) && !name.to_lowercase().contains(&query) {
                        return None;
                    }
                    Some(GemCurrencyRow {
                        title: format!("{} - {}", currency_text(&currency), name),
                        is_selected: currency == current,
                        currency,
                    })
                })
                .collect::<Vec<_>>();
            (!rows.is_empty()).then_some(GemCurrencySection { kind, rows })
        })
        .collect()
}

pub(crate) fn currency_text(currency: &Currency) -> String {
    format!("{} {}", currency.flag(), currency.as_ref())
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
    fn test_search_preserves_sections_and_marks_selection() {
        let names = HashMap::from([("USD".into(), "dólar estadounidense".into()), ("AUD".into(), "dólar australiano".into())]);
        let state = sections(Currency::USD, None, " DÓLAR ", &names, &[]);
        assert_eq!(
            state,
            vec![
                GemCurrencySection {
                    kind: GemCurrencySectionKind::Recommended,
                    rows: vec![GemCurrencyRow {
                        currency: Currency::USD,
                        title: "🇺🇸 USD - dólar estadounidense".into(),
                        is_selected: true,
                    }]
                },
                GemCurrencySection {
                    kind: GemCurrencySectionKind::All,
                    rows: vec![GemCurrencyRow {
                        currency: Currency::AUD,
                        title: "🇦🇺 AUD - dólar australiano".into(),
                        is_selected: false,
                    }]
                },
            ]
        );
        assert_eq!(sections(Currency::USD, None, " aUd ", &names, &[]), vec![state[1].clone()]);
    }

    #[test]
    fn test_empty_search_restores_all_currencies_and_no_match_has_no_sections() {
        let names = HashMap::new();
        let state = sections(Currency::GBP, Some(Currency::EUR), "", &names, &[]);
        assert_eq!(state.iter().map(|section| section.rows.len()).sum::<usize>(), Currency::iter().count());
        assert_eq!(
            state.iter().flat_map(|section| &section.rows).filter(|row| row.is_selected).map(|row| row.currency.clone()).collect::<Vec<_>>(),
            vec![Currency::GBP]
        );
        assert_eq!(sections(Currency::GBP, Some(Currency::EUR), " \n ", &names, &[]), state);
        assert_eq!(sections(Currency::GBP, Some(Currency::EUR), "not a currency", &names, &[]), vec![]);
    }

    #[test]
    fn test_recommended_currencies_lead_with_the_current_and_locale_currency_once() {
        assert_eq!(
            recommended_currencies(Currency::CHF, Some(Currency::EUR)),
            vec![Currency::CHF, Currency::EUR, Currency::USD, Currency::GBP, Currency::CNY, Currency::JPY, Currency::INR, Currency::RUB]
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
