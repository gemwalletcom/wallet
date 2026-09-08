use std::sync::Mutex;

use primitives::currency::Currency;
use primitives::{AssetId, AssetMarket, AssetPrice, FiatRate};

use super::{GemPriceStore, GemPriceUpdate};
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryPriceStore {
    pub rates: Mutex<Vec<FiatRate>>,
    pub prices: Mutex<Vec<AssetPrice>>,
    pub saved: Mutex<Vec<(Currency, Vec<GemPriceUpdate>)>>,
    pub converted: Mutex<Vec<(Currency, f64)>>,
}

#[async_trait::async_trait]
impl GemPriceStore for MemoryPriceStore {
    async fn get_prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPrice>, GemServiceError> {
        let stored = self.prices.lock().unwrap();
        Ok(asset_ids
            .iter()
            .filter_map(|asset_id| stored.iter().rev().find(|price| &price.asset_id == asset_id).cloned())
            .collect())
    }
    async fn get_rate(&self, currency: Currency) -> Result<Option<FiatRate>, GemServiceError> {
        Ok(self.rates.lock().unwrap().iter().rev().find(|rate| rate.symbol == currency).cloned())
    }
    async fn save_rates(&self, rates: Vec<FiatRate>) -> Result<(), GemServiceError> {
        self.rates.lock().unwrap().extend(rates);
        Ok(())
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
    async fn save_market(&self, _asset_id: AssetId, _market: AssetMarket) -> Result<(), GemServiceError> {
        Ok(())
    }
}
