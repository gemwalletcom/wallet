pub mod model;
pub(crate) mod rules;

use std::{collections::HashMap, sync::Arc};

use primitives::Currency;

pub use model::{GemCurrencyList, GemCurrencyRow, GemCurrencySection, GemCurrencySectionKind};

use crate::models::state::GemListPhase;
use crate::services::empty_state::{GemEmptyStateKind, empty_state};
use crate::services::error::GemServiceError;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemCurrencyService {
    prices: Arc<GemPriceService>,
}

#[uniffi::export]
impl GemCurrencyService {
    #[uniffi::constructor]
    pub fn new(prices: Arc<GemPriceService>) -> Self {
        Self { prices }
    }

    pub async fn list(&self, currency: Currency, locale: Option<Currency>, query: String, localized_names: HashMap<String, String>) -> Result<GemCurrencyList, GemServiceError> {
        let rated = self.prices.rated_currencies().await?;
        let sections = rules::sections(currency, locale, &query, &localized_names, &rated);
        Ok(GemCurrencyList {
            phase: GemListPhase::local(!sections.is_empty(), empty_state(GemEmptyStateKind::SearchResults)),
            sections,
        })
    }

    pub async fn set_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        self.prices.change_currency(currency).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::preferences::GemPreferencesService;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::price::testkit::MemoryPriceStore;
    use futures::executor::block_on;

    fn service(prices: MemoryPriceStore) -> (GemCurrencyService, Arc<MemoryPriceStore>, Arc<GemPreferencesService>) {
        let prices = Arc::new(prices);
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        (GemCurrencyService::new(Arc::new(GemPriceService::new(prices.clone(), preferences.clone()))), prices, preferences)
    }

    #[test]
    fn test_a_currency_without_a_rate_keeps_the_previous_currency() {
        let (service, prices, preferences) = service(MemoryPriceStore::default());
        let previous = preferences.get_currency();

        assert!(block_on(service.set_currency(Currency::EUR)).is_err());
        assert_eq!(preferences.get_currency(), previous);
        assert!(prices.converted.lock().unwrap().is_empty());
    }

    #[test]
    fn test_the_picker_offers_only_currencies_with_a_rate() {
        let (service, _, _) = service(MemoryPriceStore::with_rate(Currency::EUR, 0.9));

        let currencies: Vec<Currency> = block_on(service.list(Currency::GBP, None, String::new(), HashMap::new()))
            .unwrap()
            .sections
            .into_iter()
            .flat_map(|section| section.rows.into_iter().map(|row| row.currency))
            .collect();

        assert_eq!(currencies, vec![Currency::GBP, Currency::USD, Currency::EUR], "the current and the base currency stay offered");
    }

    #[test]
    fn test_a_currency_search_that_matches_nothing_shows_no_results() {
        let (service, _, _) = service(MemoryPriceStore::with_rate(Currency::EUR, 0.9));

        assert_eq!(block_on(service.list(Currency::GBP, None, String::new(), HashMap::new())).unwrap().phase, GemListPhase::Rows);
        assert_eq!(
            block_on(service.list(Currency::GBP, None, "zzz".to_string(), HashMap::new())).unwrap().phase,
            GemListPhase::Empty {
                state: empty_state(GemEmptyStateKind::SearchResults)
            }
        );
    }

    #[test]
    fn test_a_currency_with_a_rate_converts_prices_and_is_saved() {
        let (service, prices, preferences) = service(MemoryPriceStore::with_rate(Currency::EUR, 0.9));

        block_on(service.set_currency(Currency::EUR)).unwrap();

        assert_eq!(preferences.get_currency(), Currency::EUR);
        assert_eq!(*prices.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }
}
