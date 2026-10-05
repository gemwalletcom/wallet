use std::collections::HashSet;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use prices::{AssetPriceMapping, PriceProviderAssetMetadata};
use primitives::currency::Currency;
use primitives::{AssetId, AssetPriceInfo, ChartPeriod, ChartTimeframe, FiatRate, FiatRateProvider, PriceData, PriceProvider};
use storage::{AssetFilter, ChartPoint, ChartResult, DatabaseError, PriceAsset, PriceFilter, PriceProviderConfig};

use crate::prices::repository::{ChartData, Repository};

#[derive(Default)]
pub(crate) struct MemoryPricesRepository {
    fiat_rates: Vec<FiatRate>,
    charts: Vec<ChartResult>,
    price_assets: Vec<PriceAsset>,
    saved: Mutex<Vec<(Vec<PriceData>, Vec<PriceAsset>)>>,
}

impl MemoryPricesRepository {
    pub(crate) fn with_fiat_rates(self, fiat_rates: Vec<FiatRate>) -> Self {
        Self { fiat_rates, ..self }
    }

    pub(crate) fn with_charts(self, charts: Vec<ChartResult>) -> Self {
        Self { charts, ..self }
    }

    pub(crate) fn with_price_assets(self, price_assets: Vec<PriceAsset>) -> Self {
        Self { price_assets, ..self }
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

    async fn fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError> {
        Ok(self.fiat_rates.clone())
    }

    async fn fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError> {
        self.rate(&currency)
    }

    async fn prices_for_asset(&self, _asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(vec![])
    }

    async fn prices(&self, _filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError> {
        Ok(vec![])
    }

    async fn save_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        let count = price_assets.len();
        self.saved.lock().unwrap().push((prices, price_assets));
        Ok(count)
    }

    async fn price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        Ok(self.price_assets.iter().filter(|price_asset| price_ids.contains(&price_asset.price_id.to_string())).cloned().collect())
    }

    async fn asset_ids(&self, _filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(vec![])
    }

    async fn tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError> {
        Ok(tags.iter().map(|_| vec![]).collect())
    }

    async fn set_tag_asset_ids(&self, _tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError> {
        Ok(asset_ids.len())
    }

    async fn chart_data(&self, _asset_id: AssetId, currency: Currency, _period: ChartPeriod, _price_max_age: Duration) -> Result<ChartData, DatabaseError> {
        Ok(ChartData {
            base_rate: self.rate(&Currency::USD)?,
            rate: self.rate(&currency)?,
            charts: self.charts.clone(),
        })
    }

    async fn add_charts(&self, _timeframe: ChartTimeframe, points: Vec<ChartPoint>) -> Result<usize, DatabaseError> {
        Ok(points.len())
    }

    async fn aggregate_charts(&self, _timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn delete_charts(&self, _timeframe: ChartTimeframe, _before: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn update_extremes_for_price(&self, _price_id: String) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn provider_price_assets(&self, _provider: PriceProvider, _asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError> {
        Ok((self.price_assets.clone(), self.price_assets.iter().map(|price_asset| price_asset.asset_id.clone()).collect()))
    }

    async fn price_asset_ids(&self, price_id: String, _asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        Ok(self
            .price_assets
            .iter()
            .filter(|price_asset| price_asset.price_id.to_string() == price_id)
            .map(|price_asset| price_asset.asset_id.clone())
            .collect())
    }

    async fn price_mappings(&self, _provider: PriceProvider, _window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError> {
        Ok(self
            .price_assets
            .iter()
            .map(|price_asset| AssetPriceMapping::new(price_asset.asset_id.clone(), price_asset.price_id.provider_price_id.clone()))
            .collect())
    }

    async fn primary_prices(&self, _asset_ids: Vec<AssetId>, _price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError> {
        Ok(vec![])
    }

    async fn store_prices(&self, _prices: Vec<PriceData>, _price_max_age: Duration) -> Result<Vec<AssetPriceInfo>, DatabaseError> {
        Ok(vec![])
    }

    async fn update_price_changes(&self, _provider: PriceProvider, _from: NaiveDateTime, _until: NaiveDateTime) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn delete_prices(&self, _filters: Vec<PriceFilter>) -> Result<(Vec<String>, usize), DatabaseError> {
        Ok((vec![], 0))
    }

    async fn usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError> {
        Ok((vec![], HashSet::new()))
    }

    async fn price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError> {
        Ok(vec![])
    }

    async fn update_assets_metadata(&self, _metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError> {
        Ok(())
    }
}
