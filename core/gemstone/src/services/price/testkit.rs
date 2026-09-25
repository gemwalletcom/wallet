use std::sync::Mutex;

use chrono::Utc;
use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

use super::{GemPriceService, GemPriceStore, GemPriceUpdate};
use crate::services::error::GemServiceError;
use crate::services::preferences::GemPreferencesService;
use crate::services::preferences::testkit::MemoryPreferencesStore;
use std::sync::Arc;

impl GemPriceUpdate {
    pub fn mock(asset_id: AssetId, price: f64, price_change_percentage_24h: f64) -> Self {
        Self {
            asset_id,
            price,
            price_usd: price,
            price_change_percentage_24h,
            updated_at: Utc::now(),
        }
    }
}

#[derive(Default)]
pub struct MemoryPriceStore {
    pub rates: Mutex<Vec<FiatRate>>,
    pub rate_reads: Mutex<usize>,
    pub rate_writes: Mutex<Vec<Vec<FiatRate>>>,
    pub prices: Mutex<Vec<AssetPrice>>,
    pub saved: Mutex<Vec<(Currency, Vec<GemPriceUpdate>)>>,
    pub converted: Mutex<Vec<(Currency, f64)>>,
    pub rate_error: Mutex<Option<GemServiceError>>,
    pub markets: Mutex<Vec<(AssetId, AssetMarket)>>,
}

impl GemPriceService {
    pub fn mock(store: Arc<MemoryPriceStore>) -> Self {
        Self::new(store, Arc::new(GemPreferencesService::new(Arc::new(MemoryPreferencesStore::default()))))
    }
}

impl MemoryPriceStore {
    pub fn with_rate(symbol: Currency, rate: f64) -> Self {
        Self {
            rates: Mutex::new(vec![FiatRate { symbol, rate }]),
            ..Default::default()
        }
    }
}

#[async_trait::async_trait]
impl GemPriceStore for MemoryPriceStore {
    async fn get_prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemServiceError> {
        let stored = self.prices.lock().unwrap();
        Ok(asset_ids.iter().filter_map(|asset_id| stored.iter().rev().find(|price| &price.asset_id == asset_id).cloned()).collect())
    }
    async fn get_rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
        Ok(self.rates.lock().unwrap().iter().find(|rate| rate.symbol == currency).cloned())
    }
    async fn get_rates(&self) -> Result<Vec<FiatRate>, GemServiceError> {
        *self.rate_reads.lock().unwrap() += 1;
        Ok(self.rates.lock().unwrap().clone())
    }
    async fn save_rates_and_prices(&self, currency: Currency, rates: Vec<FiatRate>, conversion: Option<FiatRate>, prices: Vec<GemPriceUpdate>) -> Result<(), GemServiceError> {
        if let Some(error) = self.rate_error.lock().unwrap().clone() {
            return Err(error);
        }
        let upserted_rates: Vec<FiatRate> = self.rates.lock().unwrap().iter().filter(|stored| rates.iter().all(|rate| rate.symbol != stored.symbol)).cloned().chain(rates.clone()).collect();
        *self.rates.lock().unwrap() = upserted_rates;
        self.rate_writes.lock().unwrap().push(rates);
        if let Some(rate) = conversion {
            self.converted.lock().unwrap().push((rate.symbol, rate.rate));
        }
        if prices.is_empty() {
            return Ok(());
        }
        self.save_prices(currency, prices).await
    }
    async fn save_prices(&self, currency: Currency, prices: Vec<GemPriceUpdate>) -> Result<(), GemServiceError> {
        self.prices
            .lock()
            .unwrap()
            .extend(prices.iter().map(|price| AssetPrice::new(price.asset_id.clone(), price.price, price.price_change_percentage_24h, price.updated_at)));
        self.saved.lock().unwrap().push((currency, prices));
        Ok(())
    }
    async fn convert_prices(&self, currency: Currency, rate: f64) -> Result<(), GemServiceError> {
        self.converted.lock().unwrap().push((currency, rate));
        Ok(())
    }
    async fn save_market(&self, asset_id: AssetId, market: AssetMarket) -> Result<(), GemServiceError> {
        self.markets.lock().unwrap().push((asset_id, market));
        Ok(())
    }
}
