use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetAssociation, AssetBasic, AssetFull, AssetId, AssetPriceMetadata, ScanAddress, asset_score::AssetRank};
use storage::{AssetFilter, AssetUpdate, AssetsAddressesRepository, AssetsRepository, Database, DatabaseError, ScanAddressesRepository, WalletsRepository};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RankChange {
    pub(crate) asset_ids: Vec<AssetId>,
    pub(crate) rank: AssetRank,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError>;
    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError>;
    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError>;
    async fn asset_full(&self, asset_id: AssetId, price_max_age: Duration) -> Result<AssetFull, DatabaseError>;
    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError>;
    async fn assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    async fn wallet_assets_with_prices(&self, device_id: i32, wallet_id: i32, since: Option<NaiveDateTime>, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError>;
    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<usize, DatabaseError>;
    async fn update_assets(&self, asset_ids: Vec<AssetId>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError>;
    async fn upsert_asset_associations(&self, id: String, associations: Vec<AssetAssociation>) -> Result<usize, DatabaseError>;
    async fn add_scan_addresses(&self, addresses: Vec<ScanAddress>) -> Result<usize, DatabaseError>;
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
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError> {
        self.database.run(move |client| client.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::RankLte(rank.threshold())])).await
    }

    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        self.database
            .run(move |client| {
                changes.into_iter().try_fold(0, |count, change| {
                    let updated = client.update_assets(change.asset_ids, vec![AssetUpdate::Rank(change.rank.threshold()), AssetUpdate::IsEnabled(false)])?;
                    Ok::<_, DatabaseError>(count + updated)
                })
            })
            .await
    }

    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        self.database.run(move |client| client.get_asset(&asset_id)).await
    }

    async fn asset_full(&self, asset_id: AssetId, price_max_age: Duration) -> Result<AssetFull, DatabaseError> {
        self.database.run(move |client| client.get_asset_full(&asset_id, price_max_age)).await
    }

    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        self.database.run(move |client| client.get_assets(asset_ids)).await
    }

    async fn assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.database.run(move |client| client.get_assets_with_prices(filters, price_max_age)).await
    }

    async fn wallet_assets_with_prices(&self, device_id: i32, wallet_id: i32, since: Option<NaiveDateTime>, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.database
            .run(move |client| {
                let chain_addresses = client.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
                let asset_ids = client.get_assets_by_addresses(chain_addresses, since)?;
                if asset_ids.is_empty() {
                    return Ok(vec![]);
                }
                client.get_assets_with_prices([filters, vec![AssetFilter::Ids(asset_ids.iter().map(ToString::to_string).collect())]].concat(), price_max_age)
            })
            .await
    }

    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_assets(assets)).await
    }

    async fn update_assets(&self, asset_ids: Vec<AssetId>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.update_assets(asset_ids, updates)).await
    }

    async fn upsert_asset_associations(&self, id: String, associations: Vec<AssetAssociation>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.upsert_asset_associations(&id, associations)).await
    }

    async fn add_scan_addresses(&self, addresses: Vec<ScanAddress>) -> Result<usize, DatabaseError> {
        self.database.run(move |client| client.add_scan_addresses(addresses)).await
    }
}
