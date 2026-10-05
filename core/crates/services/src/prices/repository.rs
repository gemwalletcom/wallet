use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::currency::Currency;
use primitives::{AssetId, ChartPeriod, ChartTimeframe, FiatRate, FiatRateProvider, PriceData};
use storage::{AssetFilter, AssetsRepository, ChartPoint, ChartResult, ChartsRepository, Database, DatabaseError, FiatRepository, PriceAsset, PriceFilter, PricesRepository, TagRepository};

pub(crate) struct ChartData {
    pub(crate) base_rate: FiatRate,
    pub(crate) rate: FiatRate,
    pub(crate) charts: Vec<ChartResult>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn set_fiat_rates(&self, provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<(usize, Vec<FiatRate>), DatabaseError>;
    async fn fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError>;
    async fn fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError>;
    async fn prices_for_asset(&self, asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError>;
    async fn prices(&self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError>;
    async fn save_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError>;
    async fn price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError>;
    async fn asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError>;
    async fn set_tag_asset_ids(&self, tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError>;
    async fn chart_data(&self, asset_id: AssetId, currency: Currency, period: ChartPeriod, price_max_age: Duration) -> Result<ChartData, DatabaseError>;
    async fn add_charts(&self, timeframe: ChartTimeframe, points: Vec<ChartPoint>) -> Result<usize, DatabaseError>;
    async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
    async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    async fn update_extremes_for_price(&self, price_id: String) -> Result<usize, DatabaseError>;
}

pub(crate) struct PostgresRepository {
    database: Database,
}

impl PostgresRepository {
    pub(crate) fn new(database: Database) -> Self {
        Self { database }
    }
}

#[async_trait]
impl Repository for PostgresRepository {
    async fn set_fiat_rates(&self, provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<(usize, Vec<FiatRate>), DatabaseError> {
        self.database.run(move |client| Ok((client.set_fiat_rates(provider, rates)?, client.get_fiat_rates()?))).await
    }

    async fn fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError> {
        self.database.run(FiatRepository::get_fiat_rates).await
    }

    async fn fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError> {
        self.database.run(move |client| client.get_fiat_rate(&currency)).await
    }

    async fn prices_for_asset(&self, asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError> {
        self.database.run(move |client| client.get_prices_for_asset(&asset_id)).await
    }

    async fn prices(&self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError> {
        self.database.run(move |client| client.get_prices_by_filter(filters)).await
    }

    async fn save_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                client.add_prices(prices)?;
                client.set_prices_assets(price_assets)
            })
            .await
    }

    async fn price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        self.database.run(move |client| client.get_prices_assets_for_price_ids(price_ids)).await
    }

    async fn asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database.run(move |client| client.get_asset_ids_by_filter(filters)).await
    }

    async fn tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError> {
        self.database.run(move |client| tags.iter().map(|tag| client.get_asset_ids_for_tag(tag)).collect()).await
    }

    async fn set_tag_asset_ids(&self, tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_assets_tags_for_tag(&tag, asset_ids)).await
    }

    async fn chart_data(&self, asset_id: AssetId, currency: Currency, period: ChartPeriod, price_max_age: Duration) -> Result<ChartData, DatabaseError> {
        self.database
            .run(move |client| {
                let base_rate = client.get_fiat_rate(&Currency::USD)?;
                let rate = client.get_fiat_rate(&currency)?;
                let key = client.get_primary_price_key(&asset_id, price_max_age)?;
                Ok(ChartData {
                    base_rate,
                    rate,
                    charts: client.get_charts(&key.id(), &period)?,
                })
            })
            .await
    }

    async fn add_charts(&self, timeframe: ChartTimeframe, points: Vec<ChartPoint>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_charts(timeframe, points)).await
    }

    async fn aggregate_charts(&self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.aggregate_charts(timeframe)).await
    }

    async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.delete_charts(timeframe, before)).await
    }

    async fn update_extremes_for_price(&self, price_id: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_extremes_for_price(&price_id)).await
    }
}
