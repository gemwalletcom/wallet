use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use cacher::CacheError;
use chrono::NaiveDateTime;
use config_keys::ConfigKey;
use gem_tracing::error_with_fields;
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviders};
use primitives::currency::Currency;
use primitives::{AssetId, AssetMarketPrice, AssetPriceInfo, AssetPrices, ChartTimeframe, FiatRate, FiatRateProvider, PriceData, PriceId, PriceProvider};
use storage::{AssetFilter, AssetsRepository, ChartsRepository, Database, DatabaseError, FiatRepository, PriceAsset, PricesRepository};

use crate::ConfigCacher;
use crate::prices::{ObservedAssetsStore, PriceCacheStore};

#[derive(Clone)]
pub struct PriceClient {
    database: Database,
    config: Arc<ConfigCacher>,
    cache: Arc<dyn PriceCacheStore>,
    observed: Arc<dyn ObservedAssetsStore>,
}

impl PriceClient {
    pub fn new(database: Database, config: Arc<ConfigCacher>, cache: Arc<dyn PriceCacheStore>, observed: Arc<dyn ObservedAssetsStore>) -> Self {
        Self { database, config, cache, observed }
    }

    pub async fn set_fiat_rates(&self, provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let (count, rates) = self.database.run(move |client| -> Result<_, DatabaseError> { Ok((client.set_fiat_rates(provider, rates)?, client.get_fiat_rates()?)) }).await?;

        self.set_cache_fiat_rates(rates).await?;

        Ok(count)
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(FiatRepository::get_fiat_rates).await?)
    }

    pub async fn get_fiat_rate(&self, currency: &Currency) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        let currency = currency.clone();
        Ok(self.database.run(move |client| client.get_fiat_rate(&currency)).await?)
    }

    pub async fn get_asset_price(&self, asset_id: &AssetId, currency: &Currency) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency).await?.rate;
        let price = self.get_cache_price(asset_id).await?;
        let price_asset_id = asset_id.clone();
        let prices = self
            .database
            .run(move |client| client.get_prices_for_asset(&price_asset_id))
            .await?
            .into_iter()
            .map(|price| price.as_price().with_rate(rate))
            .collect();
        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
            prices: Some(prices),
        })
    }

    pub async fn set_cache_fiat_rates(&self, rates: Vec<FiatRate>) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cache.set_fiat_rates(&rates).await
    }

    pub async fn get_cache_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        match self.cache.fiat_rates().await? {
            Some(rates) => Ok(rates),
            None => Err(Box::new(CacheError::not_found_resource("FiatRates"))),
        }
    }

    pub async fn set_cache_prices(&self, prices: Vec<AssetPriceInfo>, ttl_seconds: i64) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.cache.set_prices(&prices, ttl_seconds).await
    }

    pub async fn get_cache_prices(&self, asset_ids: Vec<AssetId>) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        self.cache.prices(&asset_ids).await
    }

    pub async fn get_cache_price(&self, asset_id: &AssetId) -> Result<AssetPriceInfo, Box<dyn Error + Send + Sync>> {
        match self.cache.price(asset_id).await? {
            Some(price) => Ok(price),
            None => Err(Box::new(CacheError::not_found("Price", asset_id.to_string()))),
        }
    }

    pub async fn get_asset_prices(&self, currency: Currency, asset_ids: Vec<AssetId>) -> Result<AssetPrices, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(&currency).await?.rate;
        let prices = self.get_cache_prices(asset_ids).await?.into_iter().map(|x| x.as_asset_price_primitive_with_rate(rate)).collect();

        Ok(AssetPrices { currency, prices })
    }

    pub async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.aggregate_charts(timeframe)).await?)
    }

    pub async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.database.run(move |client| client.delete_charts(timeframe, before)).await?)
    }

    pub async fn track_observed_assets(&self, asset_ids: &[AssetId]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.observed.track_observed_assets(asset_ids).await
    }

    pub async fn add_prices(&self, provider: &dyn PriceAssetsProvider, mappings: Vec<AssetPriceMapping>) -> Result<Vec<PriceData>, Box<dyn Error + Send + Sync>> {
        let mappings = self.filter_existing_assets(mappings).await?;
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let prices = provider.get_prices(mappings).await?;
        if prices.is_empty() {
            return Ok(vec![]);
        }
        self.save_prices(provider.provider(), &prices).await?;
        Ok(prices.iter().map(AssetPriceFull::as_price_data).collect())
    }

    pub async fn add_prices_for_asset_id(&self, providers: &PriceProviders, asset_id: &AssetId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_id_str = asset_id.to_string();
        let mut count = 0;
        let cooldown = self.config.get_duration(ConfigKey::PriceMissingCooldown).await?;
        for provider in providers.values() {
            let kind = provider.provider();
            if self.cache.is_mapping_missing(kind, &asset_id_str).await? {
                continue;
            }
            let mappings = provider.get_mappings_for_asset_id(asset_id).await;
            if matches!(&mappings, Ok(mappings) if mappings.is_empty()) {
                self.cache.set_mapping_missing(kind, &asset_id_str, cooldown).await?;
                continue;
            }
            match self.add_prices_with_mappings(provider.as_ref(), mappings).await {
                Ok(added) => count += added,
                Err(error) => {
                    error_with_fields!("fetch prices provider failed", &*error, provider = kind.id(), asset_id = asset_id_str.as_str());
                }
            }
        }
        Ok(count)
    }

    pub async fn add_prices_for_price_id(&self, providers: &PriceProviders, price_id: &PriceId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let Some(provider) = providers.get(&price_id.provider) else {
            return Ok(0);
        };
        match self.add_prices_with_mappings(provider.as_ref(), provider.get_mappings_for_price_id(&price_id.provider_price_id).await).await {
            Ok(added) => Ok(added),
            Err(error) => {
                let kind = provider.provider();
                let price_id_str = price_id.to_string();
                error_with_fields!("fetch prices provider failed", &*error, provider = kind.id(), price_id = price_id_str.as_str());
                Ok(0)
            }
        }
    }

    async fn add_prices_with_mappings(&self, provider: &dyn PriceAssetsProvider, mappings: Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.add_prices(provider, mappings?).await?.len())
    }

    pub async fn filter_existing_assets(&self, mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
        if mappings.is_empty() {
            return Ok(vec![]);
        }
        let asset_ids = mappings.iter().map(|mapping| mapping.asset_id.to_string()).collect();
        let existing: HashSet<AssetId> = self.database.run(move |client| client.get_asset_ids_by_filter(vec![AssetFilter::Ids(asset_ids)])).await?.into_iter().collect();
        Ok(mappings.into_iter().filter(|m| existing.contains(&m.asset_id)).collect())
    }

    pub async fn save_prices(&self, provider: PriceProvider, prices: &[AssetPriceFull]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let new_prices: Vec<PriceData> = prices
            .iter()
            .map(|price| PriceData {
                id: PriceId::new(provider, price.mapping.provider_price_id.clone()),
                provider,
                ..price.as_price_data()
            })
            .collect();
        let price_assets: Vec<PriceAsset> = prices
            .iter()
            .map(|price| PriceAsset {
                asset_id: price.mapping.asset_id.clone(),
                price_id: PriceId::new(provider, price.mapping.provider_price_id.clone()),
            })
            .collect();
        self.database
            .run(move |client| -> Result<_, DatabaseError> {
                client.add_prices(new_prices)?;
                client.set_prices_assets(price_assets)
            })
            .await?;
        Ok(prices.len())
    }
}
