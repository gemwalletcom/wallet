use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use prices::{AssetPriceMapping, PriceAlertNotification, PriceAlertRules, PriceProviderAssetMetadata};
use primitives::currency::Currency;
use primitives::{AssetId, AssetPriceInfo, ChartPeriod, ChartTimeframe, FiatRate, FiatRateProvider, PriceAlert, PriceAlerts, PriceData, PriceId, PriceProvider, PriceProviderConfig};
use storage::{AssetFilter, ChartPoint, ChartResult, DatabaseError, PriceAsset, PriceFilter};

use crate::prices::repository::{ChartData, PortfolioPrice, Repository};

#[derive(Default)]
pub(crate) struct MemoryPricesRepository {
    fiat_rates: Vec<FiatRate>,
    charts: Vec<ChartResult>,
    price_assets: Vec<PriceAsset>,
    portfolio: Mutex<Vec<Option<PortfolioPrice>>>,
    saved: Mutex<Vec<(Vec<PriceData>, Vec<PriceAsset>)>>,
    providers: Vec<PriceProviderConfig>,
    provider_reads: Mutex<usize>,
    price_infos: Vec<AssetPriceInfo>,
    chart_prices: HashMap<String, f64>,
}

impl MemoryPricesRepository {
    pub(crate) fn with_providers(self, providers: Vec<PriceProviderConfig>) -> Self {
        Self { providers, ..self }
    }

    pub(crate) fn provider_reads(&self) -> usize {
        *self.provider_reads.lock().unwrap()
    }

    pub(crate) fn with_price_infos(self, price_infos: Vec<AssetPriceInfo>) -> Self {
        Self { price_infos, ..self }
    }

    pub(crate) fn with_chart_prices(self, chart_prices: HashMap<String, f64>) -> Self {
        Self { chart_prices, ..self }
    }

    pub(crate) fn stored_prices(&self) -> Vec<Vec<PriceData>> {
        self.saved.lock().unwrap().iter().map(|(prices, _)| prices.clone()).collect()
    }

    pub(crate) fn stored_price_ids(&self) -> Vec<Vec<PriceId>> {
        self.saved.lock().unwrap().iter().map(|(prices, _)| prices.iter().map(|price| price.id.clone()).collect()).collect()
    }

    pub(crate) fn with_fiat_rates(self, fiat_rates: Vec<FiatRate>) -> Self {
        Self { fiat_rates, ..self }
    }

    pub(crate) fn with_charts(self, charts: Vec<ChartResult>) -> Self {
        Self { charts, ..self }
    }

    pub(crate) fn with_price_assets(self, price_assets: Vec<PriceAsset>) -> Self {
        Self { price_assets, ..self }
    }

    pub(crate) fn with_portfolio(self, portfolio: Vec<Option<PortfolioPrice>>) -> Self {
        Self { portfolio: Mutex::new(portfolio), ..self }
    }

    fn rate(&self, currency: &Currency) -> Result<FiatRate, DatabaseError> {
        self.fiat_rates.iter().find(|rate| &rate.symbol == currency).cloned().ok_or_else(|| DatabaseError::not_found("FiatRate", currency.as_ref()))
    }
}

#[async_trait]
impl Repository for MemoryPricesRepository {
    async fn set_fiat_rates(&self, _provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<(usize, Vec<FiatRate>), DatabaseError> {
        Ok((rates.len(), rates))
    }

    async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError> {
        Ok(self.fiat_rates.clone())
    }

    async fn get_fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError> {
        self.rate(&currency)
    }

    async fn get_prices_for_asset(&self, _asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(vec![])
    }

    async fn get_prices(&self, _filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(vec![])
    }

    async fn add_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        let count = price_assets.len();
        self.saved.lock().unwrap().push((prices, price_assets));
        Ok(count)
    }

    async fn get_price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(self.price_assets.iter().filter(|price_asset| price_ids.contains(&price_asset.price_id.to_string())).cloned().collect())
    }

    async fn get_asset_ids(&self, _filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(vec![])
    }

    async fn get_tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError> {
        Ok(tags.iter().map(|_| vec![]).collect())
    }

    async fn set_tag_asset_ids(&self, _tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError> {
        Ok(asset_ids.len())
    }

    async fn get_chart_data(&self, _asset_id: AssetId, currency: Currency, _period: ChartPeriod, _price_max_age: Duration) -> Result<ChartData, DatabaseError> {
        Ok(ChartData {
            base_rate: self.rate(&Currency::USD)?,
            rate: self.rate(&currency)?,
            charts: self.charts.clone(),
        })
    }

    async fn add_charts(&self, _timeframe: ChartTimeframe, points: Vec<ChartPoint>) -> Result<usize, DatabaseError> {
        Ok(points.len())
    }

    async fn update_chart_aggregates(&self, _timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn delete_charts(&self, _timeframe: ChartTimeframe, _before: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn update_extremes_for_price(&self, _price_id: String) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn get_provider_price_assets(&self, _provider: PriceProvider, _asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError> {
        Ok((self.price_assets.clone(), self.price_assets.iter().map(|price_asset| price_asset.asset_id.clone()).collect()))
    }

    async fn get_price_asset_ids(&self, price_id: String, _asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(self
            .price_assets
            .iter()
            .filter(|price_asset| price_asset.price_id.to_string() == price_id)
            .map(|price_asset| price_asset.asset_id.clone())
            .collect())
    }

    async fn get_price_mappings(&self, _provider: PriceProvider, _window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError> {
        Ok(self
            .price_assets
            .iter()
            .map(|price_asset| AssetPriceMapping::new(price_asset.asset_id.clone(), price_asset.price_id.provider_price_id.clone()))
            .collect())
    }

    async fn get_primary_prices(&self, _asset_ids: Vec<AssetId>, _price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError> {
        Ok(vec![])
    }

    async fn get_chart_prices(&self, price_ids: Vec<String>, _from: NaiveDateTime, _until: NaiveDateTime) -> Result<HashMap<String, f64>, DatabaseError> {
        Ok(self.chart_prices.iter().filter(|(price_id, _)| price_ids.contains(price_id)).map(|(price_id, price)| (price_id.clone(), *price)).collect())
    }

    async fn set_prices(&self, prices: Vec<PriceData>) -> Result<Vec<AssetPriceInfo>, DatabaseError> {
        self.saved.lock().unwrap().push((prices, vec![]));
        Ok(self.price_infos.clone())
    }

    async fn delete_prices(&self, _filters: Vec<PriceFilter>) -> Result<(Vec<String>, usize), DatabaseError> {
        Ok((vec![], 0))
    }

    async fn get_usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError> {
        Ok((vec![], HashSet::new()))
    }

    async fn get_price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError> {
        *self.provider_reads.lock().unwrap() += 1;
        Ok(self.providers.clone())
    }

    async fn update_assets_metadata(&self, _metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn get_portfolio_prices(&self, _asset_ids: Vec<AssetId>, _period: ChartPeriod, _price_max_age: Duration) -> Result<Vec<Option<PortfolioPrice>>, DatabaseError> {
        Ok(std::mem::take(&mut *self.portfolio.lock().unwrap()))
    }

    async fn get_device_price_alerts(&self, _device_id: String, _asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, DatabaseError> {
        Ok(vec![])
    }

    async fn add_price_alerts(&self, _device_id: String, price_alerts: PriceAlerts) -> Result<usize, DatabaseError> {
        Ok(price_alerts.len())
    }

    async fn delete_price_alerts(&self, _device_id: String, ids: Vec<String>) -> Result<usize, DatabaseError> {
        Ok(ids.len())
    }

    async fn update_notified_price_alerts(&self, _rules: PriceAlertRules, _notified_before: NaiveDateTime, _now: NaiveDateTime, _price_max_age: Duration) -> Result<Vec<PriceAlertNotification>, DatabaseError> {
        Ok(vec![])
    }
}
