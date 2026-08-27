pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate, Markets, WalletId};

use crate::models::asset::asset_ids_enabled_by_default;

pub use model::GemPriceUpdate;
pub use store::GemPriceStore;

use crate::api::{GemApiClient, GemApiError};

#[derive(uniffi::Object)]
pub struct GemPriceService {
    api: Arc<GemApiClient>,
    store: Arc<dyn GemPriceStore>,
}

#[uniffi::export]
impl GemPriceService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemApiClient>, store: Arc<dyn GemPriceStore>) -> Self {
        Self { api, store }
    }

    pub async fn get_prices(&self, currency: Option<Currency>, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemApiError> {
        Ok(self.api.client.get_prices(currency, asset_ids).await?)
    }

    pub async fn get_markets(&self) -> Result<Markets, GemApiError> {
        Ok(self.api.client.get_markets().await?)
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
}

impl GemPriceService {
    pub async fn update_asset_price(&self, asset_id: AssetId, price: Option<AssetPrice>, currency: Currency) -> Result<(), GemServiceError> {
        update_prices(self.store.as_ref(), vec![price.unwrap_or_else(|| AssetPrice::empty(asset_id))], currency).await
    }

    pub async fn observable_asset_ids(&self, wallet_id: WalletId, alert_asset_ids: Vec<AssetId>) -> Result<Vec<AssetId>, GemServiceError> {
        Ok(rules::observable_asset_ids(
            self.store.get_enabled_price_asset_ids(wallet_id).await?,
            alert_asset_ids,
            asset_ids_enabled_by_default(),
        ))
    }
}

impl GemPriceService {
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
    store.save_prices(currency, rules::fiat_prices(prices, &rate)).await
}

async fn update_rates(store: &dyn GemPriceStore, rates: Vec<FiatRate>, currency: Currency) -> Result<(), GemServiceError> {
    if rates.is_empty() {
        return Ok(());
    }
    let current = rates.iter().find(|rate| rate.symbol == currency).map(|rate| rate.rate);
    store.save_rates(rates).await?;
    if let Some(rate) = current {
        store.convert_prices(currency, rate).await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use primitives::Chain;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        rates: Mutex<Vec<FiatRate>>,
        saved: Mutex<Vec<(Currency, Vec<GemPriceUpdate>)>>,
        converted: Mutex<Vec<(Currency, f64)>>,
    }

    #[async_trait::async_trait]
    impl GemPriceStore for MemoryStore {
        async fn get_enabled_price_asset_ids(&self, _wallet_id: WalletId) -> Result<Vec<AssetId>, GemServiceError> {
            Ok(vec![])
        }

        async fn get_rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
            Ok(self.rates.lock().unwrap().iter().find(|rate| rate.symbol == currency).cloned())
        }
        async fn save_rates(&self, rates: Vec<FiatRate>) -> Result<(), GemServiceError> {
            self.rates.lock().unwrap().extend(rates);
            Ok(())
        }
        async fn save_prices(&self, currency: Currency, prices: Vec<GemPriceUpdate>) -> Result<(), GemServiceError> {
            self.saved.lock().unwrap().push((currency, prices));
            Ok(())
        }
        async fn convert_prices(&self, currency: Currency, rate: f64) -> Result<(), GemServiceError> {
            self.converted.lock().unwrap().push((currency, rate));
            Ok(())
        }
        async fn save_market(&self, _asset_id: AssetId, _market: AssetMarket) -> Result<(), GemServiceError> {
            Ok(())
        }
    }

    fn store_with_rate(currency: Currency, rate: f64) -> MemoryStore {
        MemoryStore {
            rates: Mutex::new(vec![FiatRate { symbol: currency, rate }]),
            ..Default::default()
        }
    }

    fn price(value: f64) -> AssetPrice {
        AssetPrice::new(AssetId::from_chain(Chain::Solana), value, 1.5, Utc::now())
    }

    #[test]
    fn test_prices_are_converted_with_the_stored_rate() {
        let store = store_with_rate(Currency::EUR, 0.5);

        futures::executor::block_on(update_prices(&store, vec![price(100.0)], Currency::EUR)).unwrap();

        let saved = store.saved.lock().unwrap();
        assert_eq!(saved[0].0, Currency::EUR);
        assert_eq!(saved[0].1[0].price, 50.0);
        assert_eq!(saved[0].1[0].price_usd, 100.0);
    }

    #[test]
    fn test_prices_are_dropped_without_a_stored_rate_except_usd() {
        let store = MemoryStore::default();

        futures::executor::block_on(update_prices(&store, vec![price(100.0)], Currency::EUR)).unwrap();
        assert!(store.saved.lock().unwrap().is_empty());

        futures::executor::block_on(update_prices(&store, vec![price(100.0)], Currency::USD)).unwrap();
        assert_eq!(store.saved.lock().unwrap()[0].1[0].price, 100.0);
    }

    #[test]
    fn test_asset_without_price_stores_zeroed_row() {
        let store = store_with_rate(Currency::EUR, 0.5);
        let asset_id = AssetId::from_chain(Chain::Solana);

        futures::executor::block_on(update_prices(&store, vec![AssetPrice::empty(asset_id.clone())], Currency::EUR)).unwrap();

        let saved = store.saved.lock().unwrap();
        assert_eq!(saved[0].1[0].asset_id, asset_id);
        assert_eq!(saved[0].1[0].price, 0.0);
        assert_eq!(saved[0].1[0].price_usd, 0.0);
    }

    #[test]
    fn test_new_rate_for_current_currency_reconverts_stored_prices() {
        let store = MemoryStore::default();
        let rates = vec![FiatRate { symbol: Currency::EUR, rate: 0.9 }, FiatRate { symbol: Currency::GBP, rate: 0.8 }];

        futures::executor::block_on(update_rates(&store, rates.clone(), Currency::EUR)).unwrap();
        futures::executor::block_on(update_rates(&store, rates, Currency::JPY)).unwrap();

        assert_eq!(*store.converted.lock().unwrap(), vec![(Currency::EUR, 0.9)]);
    }
}
