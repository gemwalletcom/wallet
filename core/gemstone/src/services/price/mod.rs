pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

pub use model::GemPriceUpdate;
pub use store::GemPriceStore;

#[derive(uniffi::Object)]
pub struct GemPriceService {
    store: Arc<dyn GemPriceStore>,
}

#[uniffi::export]
impl GemPriceService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemPriceStore>) -> Self {
        Self { store }
    }
}

impl GemPriceService {
    pub async fn prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemServiceError> {
        Ok(self.store.get_prices(asset_ids).await?.into_iter().filter(AssetPrice::has_price).collect())
    }

    pub async fn update_prices(&self, prices: Vec<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
        update_prices(self.store.as_ref(), prices, currency).await
    }

    pub async fn update_rates(&self, rates: Vec<FiatRate>, currency: Currency) -> Result<(), GemServiceError> {
        update_rates(self.store.as_ref(), rates, currency).await
    }

    pub async fn update_market(&self, asset_id: AssetId, market: AssetMarket, currency: Currency) -> Result<(), GemServiceError> {
        let Some(rate) = self.rate(currency).await? else {
            return Ok(());
        };
        self.store.save_market(asset_id, rules::market_in_currency(market, rate.rate)).await
    }

    pub async fn change_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        let Some(rate) = rules::rate_or_base(currency.clone(), self.store.get_rate(currency.clone()).await?) else {
            return Err(GemServiceError::InvalidInput {
                msg: format!("unknown currency: {currency}"),
            });
        };
        self.store.convert_prices(currency, rate.rate).await
    }

    pub async fn update_asset_price(&self, asset_id: AssetId, price: Option<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
        update_prices(self.store.as_ref(), vec![price.unwrap_or_else(|| AssetPrice::empty(asset_id))], currency).await
    }

    pub async fn rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
        Ok(rules::rate_or_base(currency.clone(), self.store.get_rate(currency).await?))
    }
}

async fn update_prices(store: &dyn GemPriceStore, prices: Vec<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
    if prices.is_empty() {
        return Ok(());
    }
    let Some(rate) = rules::rate_or_base(currency.clone(), store.get_rate(currency.clone()).await?) else {
        return Ok(());
    };
    let stored = store.get_prices(prices.iter().map(|price| price.asset_id.clone()).collect()).await?;
    let updates = rules::changed_prices(stored, rules::fiat_prices(prices, &rate));
    if updates.is_empty() {
        return Ok(());
    }
    store.save_prices(currency, updates).await
}

async fn update_rates(store: &dyn GemPriceStore, rates: Vec<FiatRate>, currency: Currency) -> Result<(), GemServiceError> {
    if rates.is_empty() {
        return Ok(());
    }
    let changed = rules::changed_rates(store.get_rates().await?, rates);
    if changed.is_empty() {
        return Ok(());
    }
    let conversion = changed.iter().find(|rate| rate.symbol == currency).cloned();
    store.save_rates(changed, conversion).await
}

#[cfg(test)]
mod tests {
    use super::testkit::MemoryPriceStore;
    use super::*;
    use chrono::Utc;
    use primitives::Chain;

    #[test]
    fn test_prices_are_converted_with_the_stored_rate() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.5);
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        futures::executor::block_on(update_prices(&store, vec![price], Currency::EUR)).unwrap();

        let saved = store.saved.lock().unwrap();
        assert_eq!(saved[0].0, Currency::EUR);
        assert_eq!(saved[0].1[0].price, 50.0);
        assert_eq!(saved[0].1[0].price_usd, 100.0);
    }

    #[test]
    fn test_prices_are_dropped_without_a_stored_rate_except_usd() {
        let store = MemoryPriceStore::default();
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        futures::executor::block_on(update_prices(&store, vec![price.clone()], Currency::EUR)).unwrap();
        assert!(store.saved.lock().unwrap().is_empty());

        futures::executor::block_on(update_prices(&store, vec![price], Currency::USD)).unwrap();
        assert_eq!(store.saved.lock().unwrap()[0].1[0].price, 100.0);
    }

    #[test]
    fn test_asset_without_price_stores_zeroed_row() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.5);
        let asset_id = AssetId::from_chain(Chain::Solana);

        futures::executor::block_on(update_prices(&store, vec![AssetPrice::empty(asset_id.clone())], Currency::EUR)).unwrap();

        let saved = store.saved.lock().unwrap();
        assert_eq!(saved[0].1[0].asset_id, asset_id);
        assert_eq!(saved[0].1[0].price, 0.0);
        assert_eq!(saved[0].1[0].price_usd, 0.0);
    }

    #[test]
    fn test_unchanged_prices_are_not_saved() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.5);
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        futures::executor::block_on(update_prices(&store, vec![price.clone()], Currency::EUR)).unwrap();
        futures::executor::block_on(update_prices(&store, vec![price.clone()], Currency::EUR)).unwrap();
        assert_eq!(store.saved.lock().unwrap().len(), 1);

        futures::executor::block_on(update_prices(&store, vec![AssetPrice { price: 101.0, ..price }], Currency::EUR)).unwrap();
        let saved = store.saved.lock().unwrap();
        assert_eq!(saved.len(), 2);
        assert_eq!(saved[1].1[0].price_usd, 101.0);
    }

    #[test]
    fn test_new_rate_for_current_currency_reconverts_stored_prices() {
        let store = MemoryPriceStore::default();
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }];

        futures::executor::block_on(update_rates(&store, rates.clone(), Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates(&store, rates, Currency::JPY)).unwrap();

        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }

    #[test]
    fn test_failed_rate_update_can_retry_identical_rates() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.8);
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }];
        let error = GemServiceError::Store { msg: "repricing failed".into() };
        *store.rate_error.lock().unwrap() = Some(error.clone());

        assert_eq!(futures::executor::block_on(update_rates(&store, rates.clone(), Currency::EUR)), Err(error));
        assert_eq!(*store.rates.lock().unwrap(), vec![FiatRate { symbol: Currency::EUR, rate: 0.8 }]);
        assert_eq!(*store.converted.lock().unwrap(), vec![]);

        *store.rate_error.lock().unwrap() = None;
        futures::executor::block_on(update_rates(&store, rates.clone(), Currency::EUR)).unwrap();

        assert_eq!(*store.rates.lock().unwrap(), rates);
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }

    #[test]
    fn test_a_price_event_without_rates_leaves_the_rate_table_unread() {
        let store = MemoryPriceStore::default();

        futures::executor::block_on(update_rates(&store, vec![], Currency::EUR)).unwrap();

        assert_eq!(*store.rate_reads.lock().unwrap(), 0, "a tick carries no rates and must not read the table");
    }

    #[test]
    fn test_unchanged_rate_does_not_reconvert_stored_prices() {
        let store = MemoryPriceStore::default();
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }];

        futures::executor::block_on(update_rates(&store, rates.clone(), Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates(&store, rates, Currency::EUR)).unwrap();
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);

        futures::executor::block_on(update_rates(&store, vec![FiatRate { symbol: Currency::EUR, rate: 1.1 }], Currency::EUR)).unwrap();
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9), (Currency::EUR, 1.1)]);
    }

    #[test]
    fn test_only_moved_rates_are_saved() {
        let store = MemoryPriceStore::default();

        futures::executor::block_on(update_rates(&store, vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }], Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates(&store, vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }], Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates(&store, vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.7 }], Currency::EUR)).unwrap();

        assert_eq!(
            *store.rate_writes.lock().unwrap(),
            vec![
                vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }],
                vec![FiatRate { symbol: Currency::GBP, rate: 0.7 }]
            ]
        );
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }
}
