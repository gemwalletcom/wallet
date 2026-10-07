use std::collections::{HashMap, HashSet};
use std::time::Duration;

use async_trait::async_trait;
use primitives::{Asset, AssetId, NFTCollection, Perpetual, PerpetualId};
use storage::{
    AssetFilter, AssetTagLink, AssetWithMarket, AssetsRepository, AssetsUsageRanksRepository, AssetsWithPricesFilter, Database, DatabaseError, NftCollectionFilter, NftRepository, PerpetualTagLink, PerpetualsRepository, PricesRepository,
    Tag, TagRepository,
};

pub(crate) struct AssetListsIndexData {
    pub(crate) tags: Vec<Tag>,
    pub(crate) assets_tags: Vec<AssetTagLink>,
    pub(crate) searchable_asset_ids: HashSet<AssetId>,
    pub(crate) perpetuals_tags: Vec<PerpetualTagLink>,
}

pub(crate) struct PerpetualsIndexData {
    pub(crate) perpetuals: Vec<Perpetual>,
    pub(crate) list_tags: Vec<Tag>,
    pub(crate) perpetuals_tags: Vec<PerpetualTagLink>,
    pub(crate) associated_asset_ids: HashMap<PerpetualId, AssetId>,
}

#[async_trait]
pub(crate) trait Repository: Send + Sync {
    async fn assets_markets(&self, filters: Vec<AssetsWithPricesFilter>, price_max_age: Duration) -> Result<Vec<AssetWithMarket>, DatabaseError>;
    async fn usage_ranks_and_assets_tags(&self) -> Result<(Vec<(AssetId, i32)>, Vec<AssetTagLink>), DatabaseError>;
    async fn asset_lists(&self, searchable_filters: Vec<AssetFilter>) -> Result<AssetListsIndexData, DatabaseError>;
    async fn perpetuals(&self) -> Result<PerpetualsIndexData, DatabaseError>;
    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError>;
    async fn nft_collections(&self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NFTCollection>, DatabaseError>;
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
    async fn assets_markets(&self, filters: Vec<AssetsWithPricesFilter>, price_max_age: Duration) -> Result<Vec<AssetWithMarket>, DatabaseError> {
        self.database.run(move |client| client.get_assets_markets(filters, price_max_age)).await
    }

    async fn usage_ranks_and_assets_tags(&self) -> Result<(Vec<(AssetId, i32)>, Vec<AssetTagLink>), DatabaseError> {
        self.database.run(|client| Ok((client.get_all_usage_ranks()?, client.get_assets_tags()?))).await
    }

    async fn asset_lists(&self, searchable_filters: Vec<AssetFilter>) -> Result<AssetListsIndexData, DatabaseError> {
        self.database
            .run(move |client| {
                let tags = [client.get_asset_list_tags()?, client.get_perpetual_list_tags()?].concat();
                let assets_tags = client.get_assets_tags()?;
                let ids = AssetFilter::Ids(assets_tags.iter().map(|tag| tag.asset_id.to_string()).collect());
                let searchable_asset_ids = client.get_asset_ids_by_filter([vec![ids], searchable_filters].concat())?.into_iter().collect();
                let perpetuals_tags = client.get_perpetuals_tags()?;
                Ok(AssetListsIndexData {
                    tags,
                    assets_tags,
                    searchable_asset_ids,
                    perpetuals_tags,
                })
            })
            .await
    }

    async fn perpetuals(&self) -> Result<PerpetualsIndexData, DatabaseError> {
        self.database
            .run(|client| {
                Ok(PerpetualsIndexData {
                    perpetuals: client.get_perpetuals()?,
                    list_tags: client.get_perpetual_list_tags()?,
                    perpetuals_tags: client.get_perpetuals_tags()?,
                    associated_asset_ids: client.get_associated_asset_ids()?,
                })
            })
            .await
    }

    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        self.database.run(move |client| client.get_assets(asset_ids)).await
    }

    async fn nft_collections(&self, filters: Vec<NftCollectionFilter>) -> Result<Vec<NFTCollection>, DatabaseError> {
        self.database.run(move |client| client.get_nft_collections(filters)).await
    }
}
