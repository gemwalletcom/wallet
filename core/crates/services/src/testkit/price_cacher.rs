use std::collections::HashSet;
use std::error::Error;
use std::io;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use cacher::PriceCacher;
use primitives::{AssetId, AssetPriceInfo, FiatRate, PriceProvider, PriceProviderConfig};

pub(crate) struct MemoryPriceCacher {
    prices: Mutex<Vec<AssetPriceInfo>>,
    fiat_rates: Mutex<Option<Vec<FiatRate>>>,
    providers: Mutex<Option<Vec<PriceProviderConfig>>>,
    providers_unavailable: bool,
    missing_mappings: Mutex<HashSet<(PriceProvider, String)>>,
    requests: Mutex<Vec<Vec<AssetId>>>,
}

impl MemoryPriceCacher {
    pub(crate) fn new(prices: Vec<AssetPriceInfo>) -> Self {
        Self {
            prices: Mutex::new(prices),
            fiat_rates: Mutex::new(None),
            providers: Mutex::new(None),
            providers_unavailable: false,
            missing_mappings: Mutex::new(HashSet::new()),
            requests: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn requests(&self) -> Vec<Vec<AssetId>> {
        self.requests.lock().unwrap().clone()
    }

    pub(crate) fn expire_price_providers(&self) {
        *self.providers.lock().unwrap() = None;
    }

    pub(crate) fn with_unavailable_price_providers(self) -> Self {
        Self { providers_unavailable: true, ..self }
    }
}

#[async_trait]
impl PriceCacher for MemoryPriceCacher {
    async fn price_providers(&self) -> Result<Option<Vec<PriceProviderConfig>>, Box<dyn Error + Send + Sync>> {
        if self.providers_unavailable {
            return Err(io::Error::other("price providers cache unavailable").into());
        }
        Ok(self.providers.lock().unwrap().clone())
    }

    async fn set_price_providers(&self, providers: &[PriceProviderConfig]) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.providers_unavailable {
            return Err(io::Error::other("price providers cache unavailable").into());
        }
        *self.providers.lock().unwrap() = Some(providers.to_vec());
        Ok(())
    }

    async fn set_fiat_rates(&self, rates: &[FiatRate]) -> Result<(), Box<dyn Error + Send + Sync>> {
        *self.fiat_rates.lock().unwrap() = Some(rates.to_vec());
        Ok(())
    }

    async fn fiat_rates(&self) -> Result<Option<Vec<FiatRate>>, Box<dyn Error + Send + Sync>> {
        Ok(self.fiat_rates.lock().unwrap().clone())
    }

    async fn set_prices(&self, prices: &[AssetPriceInfo], _ttl: Duration) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut stored = self.prices.lock().unwrap();
        stored.retain(|price| !prices.iter().any(|update| update.asset_id == price.asset_id));
        stored.extend(prices.iter().cloned());
        Ok(prices.len())
    }

    async fn prices(&self, asset_ids: &[AssetId]) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        self.requests.lock().unwrap().push(asset_ids.to_vec());
        Ok(self.prices.lock().unwrap().iter().filter(|price| asset_ids.contains(&price.asset_id)).cloned().collect())
    }

    async fn price(&self, asset_id: &AssetId) -> Result<Option<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        Ok(self.prices.lock().unwrap().iter().find(|price| &price.asset_id == asset_id).cloned())
    }

    async fn is_mapping_missing(&self, provider: PriceProvider, asset_id: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.missing_mappings.lock().unwrap().contains(&(provider, asset_id.to_string())))
    }

    async fn set_mapping_missing(&self, provider: PriceProvider, asset_id: &str, _cooldown: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.missing_mappings.lock().unwrap().insert((provider, asset_id.to_string()));
        Ok(())
    }
}
