pub mod model;
pub(crate) mod rules;

use std::{collections::HashMap, sync::Arc};

use primitives::Currency;

pub use model::{GemCurrencyRow, GemCurrencySection, GemCurrencySectionKind};

use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::price::GemPriceService;

#[derive(uniffi::Object)]
pub struct GemCurrencyService {
    preferences: Arc<GemPreferencesService>,
    prices: Arc<GemPriceService>,
}

#[uniffi::export]
impl GemCurrencyService {
    #[uniffi::constructor]
    pub fn new(preferences: Arc<GemPreferencesService>, prices: Arc<GemPriceService>) -> Self {
        Self { preferences, prices }
    }

    pub fn sections(&self, currency: Currency, locale: Option<Currency>, query: String, localized_names: HashMap<String, String>) -> Vec<GemCurrencySection> {
        rules::sections(currency, locale, &query, &localized_names)
    }

    pub async fn set_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        if currency == self.preferences.get_currency() {
            return Ok(());
        }
        self.prices.change_currency(currency.clone()).await?;
        self.preferences.set_currency(currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::preferences::testkit::MemoryPreferencesStore;
    use crate::services::price::testkit::MemoryPriceStore;
    use futures::executor::block_on;

    fn service(prices: MemoryPriceStore) -> (GemCurrencyService, Arc<MemoryPriceStore>) {
        let prices = Arc::new(prices);
        let preferences = Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default())));
        (GemCurrencyService::new(preferences, Arc::new(GemPriceService::new(prices.clone()))), prices)
    }

    #[test]
    fn test_a_currency_without_a_rate_keeps_the_previous_currency() {
        let (service, prices) = service(MemoryPriceStore::default());
        let previous = service.preferences.get_currency();

        assert!(block_on(service.set_currency(Currency::EUR)).is_err());
        assert_eq!(service.preferences.get_currency(), previous);
        assert!(prices.converted.lock().unwrap().is_empty());
    }

    #[test]
    fn test_a_currency_with_a_rate_converts_prices_and_is_saved() {
        let (service, prices) = service(MemoryPriceStore::with_rate(Currency::EUR, 0.9));

        block_on(service.set_currency(Currency::EUR)).unwrap();

        assert_eq!(service.preferences.get_currency(), Currency::EUR);
        assert_eq!(*prices.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }
}
