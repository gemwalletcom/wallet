use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{CacheError, ObservedAssetsCacher, PriceCacher};
use chrono::NaiveDateTime;
use config_keys::ConfigKey;
use gem_tracing::{error_with_fields, warn_with_fields};
use prices::{AssetPriceFull, AssetPriceMapping, PriceAssetsProvider, PriceProviders};
use primitives::currency::Currency;
use primitives::price_provider::primary_price;
use primitives::{AssetId, AssetMarketPrice, AssetPriceInfo, AssetPrices, ChartTimeframe, FiatRate, FiatRateProvider, PriceData, PriceId, PriceProvider, PriceProviderConfig};
use storage::{AssetFilter, PriceAsset};

use super::repository::Repository;

use crate::ConfigCacher;

#[derive(Clone)]
pub struct PriceClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    cache: Arc<dyn PriceCacher>,
    observed: Arc<dyn ObservedAssetsCacher>,
}

impl PriceClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>, cache: Arc<dyn PriceCacher>, observed: Arc<dyn ObservedAssetsCacher>) -> Self {
        Self { repository, config, cache, observed }
    }

    pub async fn set_fiat_rates(&self, provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let (count, rates) = self.repository.set_fiat_rates(provider, rates).await?;

        self.cache.set_fiat_rates(&rates).await?;

        Ok(count)
    }

    async fn price_providers(&self) -> Result<Vec<PriceProviderConfig>, Box<dyn Error + Send + Sync>> {
        match self.cache.price_providers().await {
            Ok(Some(providers)) => return Ok(providers),
            Ok(None) => {}
            Err(error) => warn_with_fields!("price providers cache read failed", error = error.as_ref()),
        }
        let providers = self.repository.price_providers().await?;
        if let Err(error) = self.cache.set_price_providers(&providers).await {
            warn_with_fields!("price providers cache write failed", error = error.as_ref());
        }
        Ok(providers)
    }

    pub(crate) async fn store_prices(&self, prices: Vec<PriceData>, max_age: Duration) -> Result<Vec<AssetPriceInfo>, Box<dyn Error + Send + Sync>> {
        let prices = self.repository.store_prices(prices).await?;
        if prices.is_empty() {
            return Ok(vec![]);
        }
        let providers = self.price_providers().await?;
        let mut by_asset: HashMap<AssetId, Vec<AssetPriceInfo>> = HashMap::new();
        for price in prices {
            by_asset.entry(price.asset_id.clone()).or_default().push(price);
        }
        Ok(by_asset.into_values().filter_map(|prices| primary_price(&providers, &prices, max_age, AssetPriceInfo::as_price_primitive).cloned()).collect())
    }

    pub async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.fiat_rates().await?)
    }

    pub async fn get_fiat_rate(&self, currency: &Currency) -> Result<FiatRate, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.fiat_rate(currency.clone()).await?)
    }

    pub async fn get_asset_price(&self, asset_id: &AssetId, currency: &Currency) -> Result<AssetMarketPrice, Box<dyn Error + Send + Sync>> {
        let rate = self.get_fiat_rate(currency).await?.rate;
        let price = self.get_cache_price(asset_id).await?;
        let prices = self.repository.prices_for_asset(asset_id.clone()).await?.into_iter().map(|price| price.as_price().with_rate(rate)).collect();
        Ok(AssetMarketPrice {
            price: Some(price.as_price_primitive_with_rate(rate)),
            market: Some(price.as_market_with_rate(rate)),
            prices: Some(prices),
        })
    }

    pub async fn get_cache_fiat_rates(&self) -> Result<Vec<FiatRate>, Box<dyn Error + Send + Sync>> {
        match self.cache.fiat_rates().await? {
            Some(rates) => Ok(rates),
            None => Err(Box::new(CacheError::not_found_resource("FiatRates"))),
        }
    }

    pub async fn set_cache_prices(&self, prices: Vec<AssetPriceInfo>, ttl: Duration) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.cache.set_prices(&prices, ttl).await
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
        Ok(self.repository.aggregate_charts(timeframe).await?)
    }

    pub async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.delete_charts(timeframe, before).await?)
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
        let existing: HashSet<AssetId> = self.repository.asset_ids(vec![AssetFilter::Ids(asset_ids)]).await?.into_iter().collect();
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
        self.repository.save_prices(new_prices, price_assets).await?;
        Ok(prices.len())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration as ChronoDuration, Utc};
    use primitives::{AssetMarket, Chain, HOUR, Price};

    use super::*;
    use crate::testkit::{MemoryConfigRepository, MemoryPriceCacher, MemoryPricesRepository, UnusedObservedCacher};

    #[tokio::test]
    async fn test_price_providers() {
        let providers = vec![
            PriceProviderConfig {
                provider: PriceProvider::Coingecko,
                enabled: false,
                priority: 2,
            },
            PriceProviderConfig {
                provider: PriceProvider::Jupiter,
                enabled: true,
                priority: 0,
            },
        ];
        let repository = Arc::new(MemoryPricesRepository::default().with_providers(providers.clone()));
        let cache = Arc::new(MemoryPriceCacher::new(vec![]));
        let client = PriceClient::new(repository.clone(), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))), cache.clone(), Arc::new(UnusedObservedCacher));

        assert_eq!(client.price_providers().await.unwrap(), providers);
        assert_eq!(client.price_providers().await.unwrap(), providers);
        assert_eq!(repository.provider_reads(), 1);

        cache.expire_price_providers();
        assert_eq!(client.price_providers().await.unwrap(), providers);
        assert_eq!(repository.provider_reads(), 2);

        cache.set_price_providers(&[]).await.unwrap();
        assert_eq!(client.price_providers().await.unwrap(), vec![]);
        assert_eq!(repository.provider_reads(), 2);

        let unavailable_cache = Arc::new(MemoryPriceCacher::new(vec![]).with_unavailable_price_providers());
        let client = PriceClient::new(repository.clone(), client.config.clone(), unavailable_cache, Arc::new(UnusedObservedCacher));
        assert_eq!(client.price_providers().await.unwrap(), providers);
        assert_eq!(repository.provider_reads(), 3);
    }

    #[tokio::test]
    async fn test_store_prices() {
        let asset_id = AssetId::from_chain(Chain::Solana);
        let fresh = AssetPriceInfo {
            asset_id: asset_id.clone(),
            price: Price::new(144.0, 2.0, Utc::now(), PriceProvider::Jupiter),
            market: AssetMarket::mock(),
        };
        let repository = Arc::new(MemoryPricesRepository::default().with_price_infos(vec![
            AssetPriceInfo {
                price: Price::new(143.0, 1.0, Utc::now() - ChronoDuration::hours(2), PriceProvider::Coingecko),
                ..fresh.clone()
            },
            fresh.clone(),
            AssetPriceInfo {
                price: Price::new(145.0, 0.0, Utc::now(), PriceProvider::Pyth),
                ..fresh.clone()
            },
        ]));
        let cache = Arc::new(MemoryPriceCacher::new(vec![]));
        cache
            .set_price_providers(&[
                PriceProviderConfig {
                    provider: PriceProvider::Coingecko,
                    enabled: true,
                    priority: 0,
                },
                PriceProviderConfig {
                    provider: PriceProvider::Jupiter,
                    enabled: true,
                    priority: 2,
                },
                PriceProviderConfig {
                    provider: PriceProvider::Pyth,
                    enabled: false,
                    priority: 1,
                },
            ])
            .await
            .unwrap();
        let client = PriceClient::new(repository.clone(), Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new()))), cache, Arc::new(UnusedObservedCacher));
        let incoming = PriceData {
            id: PriceId::new(PriceProvider::Jupiter, "So11111111111111111111111111111111111111112".to_string()),
            provider: PriceProvider::Jupiter,
            provider_price_id: "So11111111111111111111111111111111111111112".to_string(),
            price: fresh.price.price,
            price_change_percentage_24h: Some(fresh.price.price_change_percentage_24h),
            last_updated_at: fresh.price.updated_at,
            ..PriceData::mock()
        };
        let selected = client.store_prices(vec![incoming.clone()], HOUR).await.unwrap();

        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].asset_id, asset_id);
        assert_eq!(selected[0].price, fresh.price);
        assert_eq!(selected[0].market.market_cap, fresh.market.market_cap);
        assert_eq!(repository.provider_reads(), 0);
        assert_eq!(repository.stored_price_ids(), vec![vec![incoming.id]]);
    }
}
