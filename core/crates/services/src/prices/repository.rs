use std::collections::{HashMap, HashSet};
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use prices::{AssetPriceMapping, PriceProviderAssetMetadata};
use primitives::currency::Currency;
use primitives::{AssetId, AssetPriceInfo, ChartPeriod, ChartTimeframe, FiatRate, FiatRateProvider, PriceData, PriceProvider};
use storage::{
    AssetFilter, AssetUpdate, AssetsLinksRepository, AssetsRepository, AssetsUsageRanksRepository, ChartFilter, ChartPoint, ChartResult, ChartsRepository, Database, DatabaseClient, DatabaseError, FiatRepository, PriceAsset, PriceFilter,
    PriceProviderConfig, PriceUpdate, PricesProvidersRepository, PricesRepository, TagRepository,
};

use super::prices_metrics_updater::price_changes;

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
    async fn provider_price_assets(&self, provider: PriceProvider, asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError>;
    async fn price_asset_ids(&self, price_id: String, asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn price_mappings(&self, provider: PriceProvider, window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError>;
    async fn primary_prices(&self, asset_ids: Vec<AssetId>, price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError>;
    async fn store_prices(&self, prices: Vec<PriceData>, price_max_age: Duration) -> Result<Vec<AssetPriceInfo>, DatabaseError>;
    async fn update_price_changes(&self, provider: PriceProvider, from: NaiveDateTime, until: NaiveDateTime) -> Result<usize, DatabaseError>;
    async fn delete_prices(&self, filters: Vec<PriceFilter>) -> Result<(Vec<String>, usize), DatabaseError>;
    async fn usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError>;
    async fn price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError>;
    async fn update_assets_metadata(&self, metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError>;
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

    async fn provider_price_assets(&self, provider: PriceProvider, asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError> {
        self.database
            .run(move |client| {
                let price_assets = client.get_prices_assets_by_provider(provider)?;
                let asset_ids = asset_ids_with_filters(client, &price_assets, asset_filters)?.into_iter().collect();
                Ok((price_assets, asset_ids))
            })
            .await
    }

    async fn price_asset_ids(&self, price_id: String, asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database
            .run(move |client| {
                let price_assets = client.get_prices_assets_for_price_ids(vec![price_id])?;
                asset_ids_with_filters(client, &price_assets, asset_filters)
            })
            .await
    }

    async fn price_mappings(&self, provider: PriceProvider, window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError> {
        self.database
            .run(move |client| {
                let prices = client.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?;
                let prices: Vec<PriceData> = match window {
                    Some((offset, limit)) => prices.into_iter().skip(offset).take(limit).collect(),
                    None => prices,
                };
                if prices.is_empty() {
                    return Ok(vec![]);
                }
                let price_ids = prices.into_iter().map(|price| price.id.to_string()).collect();
                Ok(client
                    .get_prices_assets_for_price_ids(price_ids)?
                    .into_iter()
                    .map(|mapping| AssetPriceMapping::new(mapping.asset_id, mapping.price_id.provider_price_id))
                    .collect())
            })
            .await
    }

    async fn primary_prices(&self, asset_ids: Vec<AssetId>, price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError> {
        self.database.run(move |client| client.get_primary_prices(&asset_ids, price_max_age)).await
    }

    async fn store_prices(&self, prices: Vec<PriceData>, price_max_age: Duration) -> Result<Vec<AssetPriceInfo>, DatabaseError> {
        self.database
            .run(move |client| {
                let asset_ids = client.set_prices(prices)?;
                if asset_ids.is_empty() {
                    return Ok(vec![]);
                }
                client.get_primary_price_infos(&asset_ids, price_max_age)
            })
            .await
    }

    async fn update_price_changes(&self, provider: PriceProvider, from: NaiveDateTime, until: NaiveDateTime) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                let rows = client.get_prices_by_filter(vec![PriceFilter::Provider(provider)])?;
                if rows.is_empty() {
                    return Ok(0);
                }
                let price_ids: Vec<String> = rows.iter().map(|price| price.id.to_string()).collect();
                let previous: HashMap<String, f64> = client
                    .get_charts_by_filter(vec![ChartFilter::CreatedBefore(until), ChartFilter::CreatedAfter(from), ChartFilter::PriceIds(price_ids)])?
                    .into_iter()
                    .collect();
                price_changes(&rows, &previous)
                    .into_iter()
                    .try_fold(0, |updated, (price_id, change)| Ok(updated + client.update_prices(vec![price_id], vec![PriceUpdate::PriceChangePercentage24h(change)])?))
            })
            .await
    }

    async fn delete_prices(&self, filters: Vec<PriceFilter>) -> Result<(Vec<String>, usize), DatabaseError> {
        self.database
            .run(move |client| {
                let ids: Vec<String> = client.get_prices_by_filter(filters)?.into_iter().map(|price| price.id.to_string()).collect();
                if ids.is_empty() {
                    return Ok((ids, 0));
                }
                let deleted = client.delete_prices(ids.clone())?;
                Ok((ids, deleted))
            })
            .await
    }

    async fn usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError> {
        self.database.run(|client| Ok((client.get_all_usage_ranks()?, client.get_prices_asset_ids()?.into_iter().collect()))).await
    }

    async fn price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError> {
        self.database.run(PricesProvidersRepository::get_prices_providers).await
    }

    async fn update_assets_metadata(&self, metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                for asset in metadata {
                    client.update_assets(vec![asset.asset_id.clone()], vec![AssetUpdate::Rank(asset.rank)])?;
                    client.add_assets_links(&asset.asset_id, asset.links)?;
                }
                Ok(())
            })
            .await
    }
}

fn asset_ids_with_filters(client: &mut DatabaseClient, price_assets: &[PriceAsset], asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
    let ids = AssetFilter::Ids(price_assets.iter().map(|price_asset| price_asset.asset_id.to_string()).collect());
    client.get_asset_ids_by_filter([vec![ids], asset_filters].concat())
}
