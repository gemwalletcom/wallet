use std::collections::{HashMap, HashSet};
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use prices::{AssetPriceMapping, PriceAlertNotification, PriceAlertRules, PriceProviderAssetMetadata};
use primitives::currency::Currency;
use primitives::{Asset, AssetId, AssetPriceInfo, ChartPeriod, ChartTimeframe, FiatRate, FiatRateProvider, PriceAlert, PriceAlerts, PriceData, PriceProvider, PriceProviderConfig};
use storage::{
    AssetFilter, AssetUpdate, AssetsLinksRepository, AssetsRepository, AssetsUsageRanksRepository, ChartFilter, ChartPoint, ChartResult, ChartsRepository, Database, DatabaseError, FiatRepository, PriceAlertFilter, PriceAlertUpdate,
    PriceAlertsRepository, PriceAsset, PriceFilter, PricesProvidersRepository, PricesRepository, TagRepository,
};

pub(crate) struct PortfolioPrice {
    pub(crate) asset: Asset,
    pub(crate) price: f64,
    pub(crate) charts: Vec<ChartResult>,
}

pub(crate) struct ChartData {
    pub(crate) base_rate: FiatRate,
    pub(crate) rate: FiatRate,
    pub(crate) charts: Vec<ChartResult>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn set_fiat_rates(&self, provider: FiatRateProvider, rates: Vec<FiatRate>) -> Result<(usize, Vec<FiatRate>), DatabaseError>;
    async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError>;
    async fn get_fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError>;
    async fn get_prices_for_asset(&self, asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError>;
    async fn get_prices(&self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError>;
    async fn add_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError>;
    async fn get_price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError>;
    async fn get_asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn get_tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError>;
    async fn set_tag_asset_ids(&self, tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError>;
    async fn get_chart_data(&self, asset_id: AssetId, currency: Currency, period: ChartPeriod, price_max_age: Duration) -> Result<ChartData, DatabaseError>;
    async fn add_charts(&self, timeframe: ChartTimeframe, points: Vec<ChartPoint>) -> Result<usize, DatabaseError>;
    async fn update_chart_aggregates(&self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError>;
    async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError>;
    async fn update_extremes_for_price(&self, price_id: String) -> Result<usize, DatabaseError>;
    async fn get_provider_price_assets(&self, provider: PriceProvider, asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError>;
    async fn get_price_asset_ids(&self, price_id: String, asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError>;
    async fn get_price_mappings(&self, provider: PriceProvider, window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError>;
    async fn get_primary_prices(&self, asset_ids: Vec<AssetId>, price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError>;
    async fn get_chart_prices(&self, price_ids: Vec<String>, from: NaiveDateTime, until: NaiveDateTime) -> Result<HashMap<String, f64>, DatabaseError>;
    async fn set_prices(&self, prices: Vec<PriceData>) -> Result<Vec<AssetPriceInfo>, DatabaseError>;
    async fn delete_prices(&self, filters: Vec<PriceFilter>) -> Result<(Vec<String>, usize), DatabaseError>;
    async fn get_usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError>;
    async fn get_price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError>;
    async fn update_assets_metadata(&self, metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError>;
    async fn get_portfolio_prices(&self, asset_ids: Vec<AssetId>, period: ChartPeriod, price_max_age: Duration) -> Result<Vec<Option<PortfolioPrice>>, DatabaseError>;
    async fn get_device_price_alerts(&self, device_id: String, asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, DatabaseError>;
    async fn add_price_alerts(&self, device_id: String, price_alerts: PriceAlerts) -> Result<usize, DatabaseError>;
    async fn delete_price_alerts(&self, device_id: String, ids: Vec<String>) -> Result<usize, DatabaseError>;
    async fn update_notified_price_alerts(&self, rules: PriceAlertRules, notified_before: NaiveDateTime, now: NaiveDateTime, price_max_age: Duration) -> Result<Vec<PriceAlertNotification>, DatabaseError>;
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

    async fn get_fiat_rates(&self) -> Result<Vec<FiatRate>, DatabaseError> {
        self.database.run(FiatRepository::get_fiat_rates).await
    }

    async fn get_fiat_rate(&self, currency: Currency) -> Result<FiatRate, DatabaseError> {
        self.database.run(move |client| client.get_fiat_rate(&currency)).await
    }

    async fn get_prices_for_asset(&self, asset_id: AssetId) -> Result<Vec<PriceData>, DatabaseError> {
        self.database.run(move |client| client.get_prices_for_asset(&asset_id)).await
    }

    async fn get_prices(&self, filters: Vec<PriceFilter>) -> Result<Vec<PriceData>, DatabaseError> {
        self.database.run(move |client| client.get_prices_by_filter(filters)).await
    }

    async fn add_prices(&self, prices: Vec<PriceData>, price_assets: Vec<PriceAsset>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                client.add_prices(prices)?;
                client.set_prices_assets(price_assets)
            })
            .await
    }

    async fn get_price_assets(&self, price_ids: Vec<String>) -> Result<Vec<PriceAsset>, DatabaseError> {
        self.database.run(move |client| client.get_prices_assets_for_price_ids(price_ids)).await
    }

    async fn get_asset_ids(&self, filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database.run(move |client| client.get_asset_ids_by_filter(filters)).await
    }

    async fn get_tag_asset_ids(&self, tags: Vec<String>) -> Result<Vec<Vec<AssetId>>, DatabaseError> {
        self.database.run(move |client| tags.iter().map(|tag| client.get_asset_ids_for_tag(tag)).collect()).await
    }

    async fn set_tag_asset_ids(&self, tag: String, asset_ids: Vec<AssetId>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.set_assets_tags_for_tag(&tag, asset_ids)).await
    }

    async fn get_chart_data(&self, asset_id: AssetId, currency: Currency, period: ChartPeriod, price_max_age: Duration) -> Result<ChartData, DatabaseError> {
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

    async fn update_chart_aggregates(&self, timeframe: ChartTimeframe) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_chart_aggregates(timeframe)).await
    }

    async fn delete_charts(&self, timeframe: ChartTimeframe, before: NaiveDateTime) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.delete_charts(timeframe, before)).await
    }

    async fn update_extremes_for_price(&self, price_id: String) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_extremes_for_price(&price_id)).await
    }

    async fn get_provider_price_assets(&self, provider: PriceProvider, asset_filters: Vec<AssetFilter>) -> Result<(Vec<PriceAsset>, HashSet<AssetId>), DatabaseError> {
        self.database
            .run(move |client| {
                let price_assets = client.get_prices_assets_by_provider(provider)?;
                let asset_ids = client.get_asset_ids_by_filter(price_asset_filters(&price_assets, asset_filters))?.into_iter().collect();
                Ok((price_assets, asset_ids))
            })
            .await
    }

    async fn get_price_asset_ids(&self, price_id: String, asset_filters: Vec<AssetFilter>) -> Result<Vec<AssetId>, DatabaseError> {
        self.database
            .run(move |client| {
                let price_assets = client.get_prices_assets_for_price_ids(vec![price_id])?;
                client.get_asset_ids_by_filter(price_asset_filters(&price_assets, asset_filters))
            })
            .await
    }

    async fn get_price_mappings(&self, provider: PriceProvider, window: Option<(usize, usize)>) -> Result<Vec<AssetPriceMapping>, DatabaseError> {
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

    async fn get_primary_prices(&self, asset_ids: Vec<AssetId>, price_max_age: Duration) -> Result<Vec<(AssetId, PriceData)>, DatabaseError> {
        self.database.run(move |client| client.get_primary_prices(&asset_ids, price_max_age)).await
    }

    async fn get_chart_prices(&self, price_ids: Vec<String>, from: NaiveDateTime, until: NaiveDateTime) -> Result<HashMap<String, f64>, DatabaseError> {
        self.database
            .run(move |client| {
                let filters = vec![ChartFilter::CreatedAfter(from), ChartFilter::CreatedBefore(until), ChartFilter::PriceIds(price_ids)];
                Ok(client.get_charts_by_filter(filters)?.into_iter().collect())
            })
            .await
    }

    async fn set_prices(&self, prices: Vec<PriceData>) -> Result<Vec<AssetPriceInfo>, DatabaseError> {
        self.database
            .run(move |client| {
                let asset_ids = client.set_prices(prices)?;
                if asset_ids.is_empty() {
                    return Ok(vec![]);
                }
                client.get_price_infos(&asset_ids)
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

    async fn get_usage_ranks_and_priced_assets(&self) -> Result<(Vec<(AssetId, i32)>, HashSet<AssetId>), DatabaseError> {
        self.database.run(|client| Ok((client.get_all_usage_ranks()?, client.get_prices_asset_ids()?.into_iter().collect()))).await
    }

    async fn get_price_providers(&self) -> Result<Vec<PriceProviderConfig>, DatabaseError> {
        self.database.run(PricesProvidersRepository::get_prices_providers).await
    }

    async fn update_assets_metadata(&self, metadata: Vec<PriceProviderAssetMetadata>) -> Result<(), DatabaseError> {
        self.database
            .run(move |client| {
                for asset in metadata {
                    client.update_assets(vec![AssetFilter::Ids(vec![asset.asset_id.to_string()])], vec![AssetUpdate::Rank(asset.rank)])?;
                    client.add_assets_links(&asset.asset_id, asset.links)?;
                }
                Ok(())
            })
            .await
    }

    async fn get_portfolio_prices(&self, asset_ids: Vec<AssetId>, period: ChartPeriod, price_max_age: Duration) -> Result<Vec<Option<PortfolioPrice>>, DatabaseError> {
        self.database
            .run(move |client| {
                let assets: HashMap<AssetId, Asset> = client.get_assets(asset_ids.clone())?.into_iter().map(|asset| (asset.id.clone(), asset)).collect();
                let prices: HashMap<AssetId, PriceData> = client.get_primary_prices(&asset_ids, price_max_age)?.into_iter().collect();
                Ok(asset_ids
                    .iter()
                    .map(|asset_id| {
                        let asset = assets.get(asset_id)?.clone();
                        let price = prices.get(asset_id)?;
                        let charts = client.get_charts(&price.id.id(), &period).unwrap_or_default();
                        Some(PortfolioPrice { asset, price: price.price, charts })
                    })
                    .collect())
            })
            .await
    }

    async fn get_device_price_alerts(&self, device_id: String, asset_id: Option<AssetId>) -> Result<Vec<PriceAlert>, DatabaseError> {
        self.database
            .run(move |client| Ok(client.get_price_alerts_for_device_id(&device_id, asset_id.as_ref())?.into_iter().map(|row| row.price_alert).collect()))
            .await
    }

    async fn add_price_alerts(&self, device_id: String, price_alerts: PriceAlerts) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_price_alerts(&device_id, price_alerts)).await
    }

    async fn delete_price_alerts(&self, device_id: String, ids: Vec<String>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.delete_price_alerts(&device_id, ids)).await
    }

    async fn update_notified_price_alerts(&self, rules: PriceAlertRules, notified_before: NaiveDateTime, now: NaiveDateTime, price_max_age: Duration) -> Result<Vec<PriceAlertNotification>, DatabaseError> {
        self.database
            .run(move |client| {
                let price_alerts = client.get_price_alerts(notified_before, price_max_age)?;
                let rates = client.get_fiat_rates()?;
                let mut notifications = Vec::new();
                let mut notified_ids = HashSet::new();
                for (price_alert, price_data, device) in price_alerts {
                    let Some(trigger) = rules.evaluate(&price_alert, &device, &price_data, &rates) else {
                        continue;
                    };
                    notified_ids.insert(price_alert.id());
                    let asset = client.get_asset(&price_alert.asset_id)?;
                    notifications.push(PriceAlertNotification::new(device, asset, price_alert, &price_data, trigger));
                }
                client.update_price_alerts(vec![PriceAlertFilter::Ids(notified_ids.into_iter().collect())], vec![PriceAlertUpdate::LastNotifiedAt(now)])?;
                Ok(notifications)
            })
            .await
    }
}

fn price_asset_filters(price_assets: &[PriceAsset], asset_filters: Vec<AssetFilter>) -> Vec<AssetFilter> {
    let ids = AssetFilter::Ids(price_assets.iter().map(|price_asset| price_asset.asset_id.to_string()).collect());
    [vec![ids], asset_filters].concat()
}
