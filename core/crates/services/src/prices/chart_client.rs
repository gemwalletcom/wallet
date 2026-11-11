use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use cacher::{ChartCandlesCacher, RateLimitCacher};
use config_keys::{ConfigKey, RateLimitKey};
use prices::{PriceAssetsProvider, PriceProviders};
use primitives::{AssetId, ChartCandleStick, ChartCandleStickVecExt, ChartPeriod, ChartValue, PriceData, RequestError, currency::Currency};

use super::repository::{ChartData, Repository};

use crate::ConfigCacher;

fn candles_max_age(period: ChartPeriod) -> Duration {
    match period {
        ChartPeriod::Hour | ChartPeriod::Day => Duration::from_secs(5 * 60),
        ChartPeriod::Week | ChartPeriod::Month => Duration::from_secs(30 * 60),
        ChartPeriod::Year | ChartPeriod::All => Duration::from_secs(6 * 60 * 60),
    }
}

#[derive(Clone)]
pub struct ChartClient {
    repository: Arc<dyn Repository>,
    config: Arc<ConfigCacher>,
    providers: Arc<PriceProviders>,
    cacher: Arc<dyn ChartCandlesCacher>,
    rate_limiter: Arc<dyn RateLimitCacher>,
}

impl ChartClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: Arc<ConfigCacher>, providers: Arc<PriceProviders>, cacher: Arc<dyn ChartCandlesCacher>, rate_limiter: Arc<dyn RateLimitCacher>) -> Self {
        Self {
            repository,
            config,
            providers,
            cacher,
            rate_limiter,
        }
    }

    pub async fn has_candles(&self, asset_id: &AssetId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.candles_source(asset_id).await?.is_some())
    }

    pub async fn get_candles(&self, asset_id: &AssetId, period: ChartPeriod, ip_address: &str) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Send + Sync>> {
        let Some((provider, price)) = self.candles_source(asset_id).await? else {
            return Ok(vec![]);
        };
        let price_id = price.id.id();
        let fetch = async move {
            self.consume_candles_limit(ip_address).await?;
            provider.get_candles(&price.provider_price_id, period).await
        };
        let candles = self.cacher.get_or_fetch_candles(&price_id, period, candles_max_age(period), Box::pin(fetch)).await?;
        Ok(candles.merged(self.config.get_usize(ConfigKey::PriceChartsMaxCandles).await?))
    }

    async fn consume_candles_limit(&self, ip_address: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        let key = RateLimitKey::ChartCandlesPerIpLimit;
        match self.rate_limiter.consume(key, ip_address, self.config.get_rate_limit(key).await?).await? {
            true => Ok(()),
            false => Err(RequestError::LimitReached.into()),
        }
    }

    async fn candles_source(&self, asset_id: &AssetId) -> Result<Option<(Arc<dyn PriceAssetsProvider>, PriceData)>, Box<dyn Error + Send + Sync>> {
        let prices = self.repository.get_prices_for_asset(asset_id.clone()).await?;
        Ok(prices.into_iter().filter(|price| price.provider.has_candles()).find_map(|price| Some((self.providers.get(&price.provider)?.clone(), price))))
    }

    pub async fn get_charts_prices(&self, asset_id: &AssetId, period: ChartPeriod, currency: &Currency) -> Result<Vec<ChartValue>, Box<dyn Error + Send + Sync>> {
        let primary_price_max_age = self.config.get_duration(ConfigKey::PricePrimaryMaxAge).await?;
        let ChartData { base_rate, rate, charts } = self.repository.get_chart_data(asset_id.clone(), currency.clone(), period, primary_price_max_age).await?;
        let rate_multiplier = rate.multiplier(base_rate.rate);
        Ok(charts
            .into_iter()
            .map(|(ts, price)| ChartValue {
                timestamp: ts.and_utc().timestamp() as i32,
                value: (price * rate_multiplier) as f32,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{DateTime, TimeDelta};
    use config_keys::RateLimitWindow;
    use prices::{AssetPriceFull, AssetPriceMapping, PriceProvider, PriceProviderAsset};
    use primitives::{AssetId, Chain, FiatRate};

    use super::*;
    use crate::testkit::{MemoryChartCandlesCacher, MemoryConfigRepository, MemoryPricesRepository, MemoryRateLimiter};

    #[tokio::test]
    async fn test_get_charts_prices_converts_to_currency() {
        let at = DateTime::from_timestamp(1_700_000_000, 0).unwrap().naive_utc();
        let repository = MemoryPricesRepository::default()
            .with_fiat_rates(vec![FiatRate { symbol: Currency::USD, rate: 1.0 }, FiatRate { symbol: Currency::EUR, rate: 0.5 }])
            .with_charts(vec![(at, 100.0)]);
        let config = Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new())));
        let client = ChartClient::new(Arc::new(repository), config, Arc::new(HashMap::new()), Arc::new(MemoryChartCandlesCacher::default()), Arc::new(MemoryRateLimiter::default()));

        let values = client.get_charts_prices(&AssetId::from_chain(Chain::Bitcoin), ChartPeriod::Day, &Currency::EUR).await.unwrap();

        assert_eq!(values, vec![ChartValue { timestamp: 1_700_000_000, value: 50.0 }]);
    }

    const IP: &str = "198.51.100.7";

    fn candle() -> ChartCandleStick {
        ChartCandleStick {
            date: DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            open: 1.0,
            high: 3.0,
            low: 0.5,
            close: 2.0,
            volume: 0.0,
        }
    }

    struct StubProvider {
        fails: bool,
        candles: Vec<ChartCandleStick>,
        requests: Mutex<Vec<(String, ChartPeriod)>>,
    }

    impl Default for StubProvider {
        fn default() -> Self {
            Self {
                fails: false,
                candles: vec![candle()],
                requests: Mutex::default(),
            }
        }
    }

    #[async_trait]
    impl PriceAssetsProvider for StubProvider {
        fn provider(&self) -> PriceProvider {
            PriceProvider::Coingecko
        }
        async fn get_assets(&self, _limit: usize) -> Result<Vec<PriceProviderAsset>, Box<dyn Error + Send + Sync>> {
            Ok(vec![])
        }
        async fn get_mappings_for_asset_id(&self, _asset_id: &AssetId) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
            Ok(vec![])
        }
        async fn get_mappings_for_price_id(&self, _provider_price_id: &str) -> Result<Vec<AssetPriceMapping>, Box<dyn Error + Send + Sync>> {
            Ok(vec![])
        }
        async fn get_prices(&self, _mappings: Vec<AssetPriceMapping>) -> Result<Vec<AssetPriceFull>, Box<dyn Error + Send + Sync>> {
            Ok(vec![])
        }
        async fn get_candles(&self, provider_price_id: &str, period: ChartPeriod) -> Result<Vec<ChartCandleStick>, Box<dyn Error + Send + Sync>> {
            self.requests.lock().unwrap().push((provider_price_id.to_string(), period));
            if self.fails {
                return Err("upstream failed".into());
            }
            Ok(self.candles.clone())
        }
    }

    fn candles_client(provider: Arc<StubProvider>, prices: Vec<PriceData>) -> ChartClient {
        let config = Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new())));
        let providers: PriceProviders = HashMap::from([(PriceProvider::Coingecko, provider as Arc<dyn PriceAssetsProvider>)]);
        ChartClient::new(
            Arc::new(MemoryPricesRepository::default().with_prices(prices)),
            config,
            Arc::new(providers),
            Arc::new(MemoryChartCandlesCacher::default()),
            Arc::new(MemoryRateLimiter::default()),
        )
    }

    fn pyth_price() -> PriceData {
        PriceData {
            provider: PriceProvider::Pyth,
            ..PriceData::mock()
        }
    }

    #[tokio::test]
    async fn test_an_asset_has_candles_when_a_provider_with_candles_prices_it() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let provider = Arc::new(StubProvider::default());

        assert!(candles_client(provider.clone(), vec![PriceData::mock()]).has_candles(&asset_id).await.unwrap());
        assert!(!candles_client(provider.clone(), vec![pyth_price()]).has_candles(&asset_id).await.unwrap());
        assert!(!candles_client(provider, vec![]).has_candles(&asset_id).await.unwrap());
    }

    #[tokio::test]
    async fn test_candles_come_from_the_provider_once_per_period() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let provider = Arc::new(StubProvider::default());
        let price = PriceData::mock();
        let client = candles_client(provider.clone(), vec![price.clone()]);

        assert_eq!(client.get_candles(&asset_id, ChartPeriod::Week, IP).await.unwrap(), vec![candle()]);
        assert_eq!(client.get_candles(&asset_id, ChartPeriod::Week, IP).await.unwrap(), vec![candle()]);
        client.get_candles(&asset_id, ChartPeriod::Year, IP).await.unwrap();

        assert_eq!(*provider.requests.lock().unwrap(), vec![(price.provider_price_id.clone(), ChartPeriod::Week), (price.provider_price_id, ChartPeriod::Year)]);
    }

    #[tokio::test]
    async fn test_candles_merge_down_to_the_configured_limit() {
        let provider = Arc::new(StubProvider {
            candles: ChartCandleStick::mock_series(DateTime::UNIX_EPOCH, TimeDelta::hours(4), 6),
            ..Default::default()
        });
        let providers: PriceProviders = HashMap::from([(PriceProvider::Coingecko, provider as Arc<dyn PriceAssetsProvider>)]);
        let config = MemoryConfigRepository::new().with_value(ConfigKey::PriceChartsMaxCandles.as_ref(), "2");
        let client = ChartClient::new(
            Arc::new(MemoryPricesRepository::default().with_prices(vec![PriceData::mock()])),
            Arc::new(ConfigCacher::new(Arc::new(config))),
            Arc::new(providers),
            Arc::new(MemoryChartCandlesCacher::default()),
            Arc::new(MemoryRateLimiter::default()),
        );

        let candles = client.get_candles(&AssetId::from_chain(Chain::Bitcoin), ChartPeriod::Month, IP).await.unwrap();

        assert_eq!(candles.iter().map(|candle| candle.date).collect::<Vec<_>>(), vec![DateTime::UNIX_EPOCH, DateTime::UNIX_EPOCH + TimeDelta::hours(12)]);
    }

    #[tokio::test]
    async fn test_an_asset_without_a_candle_source_has_no_candles() {
        let client = candles_client(Arc::new(StubProvider::default()), vec![pyth_price()]);

        assert!(client.get_candles(&AssetId::from_chain(Chain::Bitcoin), ChartPeriod::Day, IP).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_an_address_stops_reaching_the_provider_at_its_limit() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let provider = Arc::new(StubProvider { fails: true, ..Default::default() });
        let client = candles_client(provider.clone(), vec![PriceData::mock()]);
        let limit = client.config.get_rate_limit(RateLimitKey::ChartCandlesPerIpLimit).await.unwrap().get(RateLimitWindow::Minute);

        for _ in 0..limit {
            assert_eq!(client.get_candles(&asset_id, ChartPeriod::Day, IP).await.unwrap_err().to_string(), "upstream failed");
        }
        let limited = client.get_candles(&asset_id, ChartPeriod::Day, IP).await.unwrap_err();
        assert!(matches!(limited.downcast_ref::<RequestError>(), Some(RequestError::LimitReached)));
        assert_eq!(provider.requests.lock().unwrap().len() as i64, limit);

        assert_eq!(
            client.get_candles(&asset_id, ChartPeriod::Day, "203.0.113.8").await.unwrap_err().to_string(),
            "upstream failed",
            "another address keeps its own limit"
        );
    }

    #[tokio::test]
    async fn test_cached_candles_spend_no_limit() {
        let asset_id = AssetId::from_chain(Chain::Bitcoin);
        let rate_limiter = Arc::new(MemoryRateLimiter::default());
        let config = Arc::new(ConfigCacher::new(Arc::new(MemoryConfigRepository::new())));
        let provider = Arc::new(StubProvider::default());
        let providers: PriceProviders = HashMap::from([(PriceProvider::Coingecko, provider as Arc<dyn PriceAssetsProvider>)]);
        let client = ChartClient::new(
            Arc::new(MemoryPricesRepository::default().with_prices(vec![PriceData::mock()])),
            config,
            Arc::new(providers),
            Arc::new(MemoryChartCandlesCacher::default()),
            rate_limiter.clone(),
        );

        for _ in 0..3 {
            client.get_candles(&asset_id, ChartPeriod::Day, IP).await.unwrap();
        }

        assert_eq!(rate_limiter.consumed(IP), 1);
    }
}
