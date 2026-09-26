pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use futures::lock::Mutex;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

pub use model::GemPriceUpdate;
pub use store::GemPriceStore;

#[derive(uniffi::Object)]
pub struct GemPriceService {
    store: Arc<dyn GemPriceStore>,
    preferences: Arc<GemPreferencesService>,
    writes: Mutex<()>,
}

#[uniffi::export]
impl GemPriceService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemPriceStore>, preferences: Arc<GemPreferencesService>) -> Self {
        Self { store, preferences, writes: Mutex::new(()) }
    }
}

impl GemPriceService {
    pub async fn change_currency(&self, currency: Currency) -> Result<(), GemServiceError> {
        let _writes = self.writes.lock().await;
        let previous = self.preferences.get_currency();
        if currency == previous {
            return Ok(());
        }
        let Some(rate) = self.rate(currency.clone()).await? else {
            return Err(GemServiceError::InvalidInput {
                msg: format!("unknown currency: {currency}"),
            });
        };
        self.store.convert_prices(currency.clone(), rate.rate).await?;
        self.preferences.set_currency(currency)
    }

    pub async fn rated_currencies(&self) -> Result<Vec<Currency>, GemServiceError> {
        Ok(self.store.get_rates().await?.into_iter().map(|rate| rate.symbol).collect())
    }

    pub async fn prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemServiceError> {
        Ok(self.store.get_prices(asset_ids).await?.into_iter().filter(AssetPrice::has_price).collect())
    }

    pub async fn update_prices(&self, prices: Vec<AssetPrice>) -> Result<(), GemServiceError> {
        let _writes = self.writes.lock().await;
        update_prices(self.store.as_ref(), prices, self.preferences.get_currency()).await
    }

    pub async fn update_rates_and_prices(&self, rates: Vec<FiatRate>, prices: Vec<AssetPrice>) -> Result<(), GemServiceError> {
        let _writes = self.writes.lock().await;
        update_rates_and_prices(self.store.as_ref(), rates, prices, self.preferences.get_currency()).await
    }

    pub async fn update_asset_price(&self, asset_id: AssetId, price: Option<AssetPrice>) -> Result<(), GemServiceError> {
        self.update_prices(vec![price.unwrap_or_else(|| AssetPrice::empty(asset_id))]).await
    }

    pub async fn update_market(&self, asset_id: AssetId, market: AssetMarket) -> Result<(), GemServiceError> {
        self.store.save_market(asset_id, market).await
    }

    pub async fn market_in_currency(&self, market: AssetMarket, currency: Currency) -> Result<Option<AssetMarket>, GemServiceError> {
        Ok(self.rate(currency).await?.map(|rate| rules::market_in_currency(market, rate.rate)))
    }

    pub async fn rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
        Ok(rules::rate_or_base(currency.clone(), self.store.get_rate(currency).await?))
    }
}

async fn update_prices(store: &dyn GemPriceStore, prices: Vec<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
    let updates = price_updates(store, prices, currency.clone(), None).await?;
    if updates.is_empty() {
        return Ok(());
    }
    store.save_prices(currency, updates).await
}

async fn update_rates_and_prices(store: &dyn GemPriceStore, rates: Vec<FiatRate>, prices: Vec<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
    let rates = match rates.is_empty() {
        true => vec![],
        false => rules::changed_rates(store.get_rates().await?, rates),
    };
    let conversion = rates.iter().find(|rate| rate.symbol == currency).cloned();
    let updates = price_updates(store, prices, currency.clone(), conversion.clone()).await?;
    if rates.is_empty() && updates.is_empty() {
        return Ok(());
    }
    store.save_rates_and_prices(currency, rates, conversion, updates).await
}

async fn price_updates(store: &dyn GemPriceStore, prices: Vec<AssetPrice>, currency: Currency, conversion: Option<FiatRate>) -> Result<Vec<GemPriceUpdate>, GemServiceError> {
    if prices.is_empty() {
        return Ok(vec![]);
    }
    let rate = match conversion {
        Some(rate) => rate,
        None => match rules::rate_or_base(currency.clone(), store.get_rate(currency).await?) {
            Some(rate) => rate,
            None => return Ok(vec![]),
        },
    };
    let stored = store.get_prices(prices.iter().map(|price| price.asset_id.clone()).collect()).await?;
    Ok(rules::changed_prices(stored, rules::fiat_prices(prices, &rate)))
}

#[cfg(test)]
mod tests {
    use super::testkit::MemoryPriceStore;
    use super::*;
    use chrono::Utc;
    use primitives::Chain;

    fn rates() -> MemoryPriceStore {
        MemoryPriceStore {
            rates: std::sync::Mutex::new(vec![FiatRate { symbol: Currency::EUR, rate: 0.5 }, FiatRate { symbol: Currency::JPY, rate: 150.0 }]),
            ..Default::default()
        }
    }

    #[test]
    fn test_a_refresh_that_started_before_a_currency_switch_commits_in_the_new_currency() {
        let store = Arc::new(rates());
        let service = GemPriceService::mock(store.clone());
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        futures::executor::block_on(service.change_currency(Currency::EUR)).unwrap();
        futures::executor::block_on(service.change_currency(Currency::JPY)).unwrap();
        futures::executor::block_on(service.update_prices(vec![price.clone()])).unwrap();
        futures::executor::block_on(service.update_asset_price(price.asset_id.clone(), Some(AssetPrice { price: 200.0, ..price }))).unwrap();

        let saved = store.saved.lock().unwrap();
        assert_eq!(
            saved.iter().map(|(currency, updates)| (currency.clone(), updates[0].price)).collect::<Vec<_>>(),
            vec![(Currency::JPY, 15_000.0), (Currency::JPY, 30_000.0)]
        );
        assert_eq!(service.preferences.get_currency(), Currency::JPY);
    }

    #[test]
    fn test_a_failed_currency_switch_keeps_the_current_currency() {
        let store = Arc::new(MemoryPriceStore::with_rate(Currency::EUR, 0.5));
        let service = GemPriceService::mock(store.clone());
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        assert!(futures::executor::block_on(service.change_currency(Currency::JPY)).is_err());
        futures::executor::block_on(service.update_prices(vec![price])).unwrap();

        assert_eq!(store.saved.lock().unwrap()[0].0, Currency::USD);
        assert!(store.converted.lock().unwrap().is_empty());
    }

    #[test]
    fn test_a_market_is_saved_in_usd_and_shown_in_the_current_currency() {
        let store = Arc::new(rates());
        let service = GemPriceService::mock(store.clone());
        let market = AssetMarket {
            market_cap: Some(1_000.0),
            circulating_supply: Some(10.0),
            ..Default::default()
        };

        futures::executor::block_on(service.change_currency(Currency::EUR)).unwrap();
        futures::executor::block_on(service.update_market(AssetId::from_chain(Chain::Solana), market.clone())).unwrap();

        assert_eq!(store.markets.lock().unwrap()[0].1.market_cap, Some(1_000.0));
        let shown = futures::executor::block_on(service.market_in_currency(market.clone(), Currency::EUR)).unwrap().unwrap();
        assert_eq!(shown.market_cap, Some(500.0));
        assert_eq!(shown.circulating_supply, Some(10.0));
        assert!(futures::executor::block_on(service.market_in_currency(market, Currency::GBP)).unwrap().is_none());
    }

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

        futures::executor::block_on(update_rates_and_prices(&store, rates.clone(), vec![], Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates_and_prices(&store, rates, vec![], Currency::JPY)).unwrap();

        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }

    #[test]
    fn test_failed_rate_update_can_retry_identical_rates() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.8);
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }];
        let error = GemServiceError::Store { msg: "repricing failed".into() };
        *store.rate_error.lock().unwrap() = Some(error.clone());

        assert_eq!(futures::executor::block_on(update_rates_and_prices(&store, rates.clone(), vec![], Currency::EUR)), Err(error));
        assert_eq!(*store.rates.lock().unwrap(), vec![FiatRate { symbol: Currency::EUR, rate: 0.8 }]);
        assert_eq!(*store.converted.lock().unwrap(), vec![]);

        *store.rate_error.lock().unwrap() = None;
        futures::executor::block_on(update_rates_and_prices(&store, rates.clone(), vec![], Currency::EUR)).unwrap();

        assert_eq!(*store.rates.lock().unwrap(), rates);
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }

    #[test]
    fn test_a_tick_with_a_new_rate_writes_its_prices_at_that_rate_in_the_same_save() {
        let store = MemoryPriceStore::with_rate(Currency::EUR, 0.5);
        let price = AssetPrice::new(AssetId::from_chain(Chain::Solana), 100.0, 1.5, Utc::now());

        futures::executor::block_on(update_rates_and_prices(&store, vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }], vec![price], Currency::EUR)).unwrap();

        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
        let saved = store.saved.lock().unwrap();
        assert_eq!(saved.len(), 1, "one tick is one save");
        assert_eq!(saved[0].1[0].price, 90.0, "the price is converted at the rate that arrived with it, not the stored one");
    }

    #[test]
    fn test_a_price_event_without_rates_leaves_the_rate_table_unread() {
        let store = MemoryPriceStore::default();

        futures::executor::block_on(update_rates_and_prices(&store, vec![], vec![], Currency::EUR)).unwrap();

        assert_eq!(*store.rate_reads.lock().unwrap(), 0, "a tick carries no rates and must not read the table");
    }

    #[test]
    fn test_unchanged_rate_does_not_reconvert_stored_prices() {
        let store = MemoryPriceStore::default();
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }];

        futures::executor::block_on(update_rates_and_prices(&store, rates.clone(), vec![], Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates_and_prices(&store, rates, vec![], Currency::EUR)).unwrap();
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);

        futures::executor::block_on(update_rates_and_prices(&store, vec![FiatRate { symbol: Currency::EUR, rate: 1.1 }], vec![], Currency::EUR)).unwrap();
        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9), (Currency::EUR, 1.1)]);
    }

    #[test]
    fn test_only_moved_rates_are_saved() {
        let store = MemoryPriceStore::default();

        futures::executor::block_on(update_rates_and_prices(
            &store,
            vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }],
            vec![],
            Currency::EUR,
        ))
        .unwrap();
        futures::executor::block_on(update_rates_and_prices(
            &store,
            vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }],
            vec![],
            Currency::EUR,
        ))
        .unwrap();
        futures::executor::block_on(update_rates_and_prices(
            &store,
            vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.7 }],
            vec![],
            Currency::EUR,
        ))
        .unwrap();

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
