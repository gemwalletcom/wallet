use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetAssociation, AssetBalance, AssetBasic, AssetFull, AssetId, AssetPriceMetadata, Chain, ChainAddress, ListId, Perpetual, ScanAddress, asset_score::AssetRank, fiat_assets::AssetCatalog};
use storage::{AssetFilter, AssetUpdate, DatabaseError, Tag};

use crate::assets::repository::{RankChange, Repository, TokenAddressesUpdate};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ListAssets {
    pub(crate) tag_id: String,
    pub(crate) asset_ids: Vec<AssetId>,
    pub(crate) is_new_tag: bool,
}

pub(crate) struct MemoryAssetRepository {
    candidates: Vec<AssetBasic>,
    ranks: Mutex<Vec<AssetRank>>,
    changes: Mutex<Vec<RankChange>>,
    price_queries: Mutex<Vec<(Vec<AssetFilter>, Duration)>>,
    updates: Mutex<Vec<(Vec<AssetId>, Vec<AssetUpdate>)>>,
    added: Mutex<Vec<AssetBasic>>,
    tags: Vec<Tag>,
    list_assets: Mutex<Vec<ListAssets>>,
    catalog: AssetCatalog,
    catalog_reads: AtomicUsize,
}

impl MemoryAssetRepository {
    pub(crate) fn new(candidates: Vec<AssetBasic>) -> Self {
        Self {
            candidates,
            ranks: Mutex::new(Vec::new()),
            changes: Mutex::new(Vec::new()),
            price_queries: Mutex::new(Vec::new()),
            updates: Mutex::new(Vec::new()),
            added: Mutex::new(Vec::new()),
            tags: Vec::new(),
            list_assets: Mutex::new(Vec::new()),
            catalog: AssetCatalog::new(vec![], vec![], vec![]),
            catalog_reads: AtomicUsize::new(0),
        }
    }

    pub(crate) fn with_catalog(self, catalog: AssetCatalog) -> Self {
        Self { catalog, ..self }
    }

    pub(crate) fn catalog_reads(&self) -> usize {
        self.catalog_reads.load(Ordering::Relaxed)
    }

    pub(crate) fn with_tags(self, tags: Vec<Tag>) -> Self {
        Self { tags, ..self }
    }

    pub(crate) fn list_assets(&self) -> Vec<ListAssets> {
        self.list_assets.lock().unwrap().clone()
    }

    pub(crate) fn ranks(&self) -> Vec<AssetRank> {
        self.ranks.lock().unwrap().clone()
    }

    pub(crate) fn changes(&self) -> Vec<RankChange> {
        self.changes.lock().unwrap().clone()
    }

    pub(crate) fn price_queries(&self) -> Vec<(Vec<AssetFilter>, Duration)> {
        self.price_queries.lock().unwrap().clone()
    }

    pub(crate) fn updates(&self) -> Vec<(Vec<AssetId>, Vec<AssetUpdate>)> {
        self.updates.lock().unwrap().clone()
    }

    fn find(&self, asset_id: &AssetId) -> Result<&AssetBasic, DatabaseError> {
        self.candidates.iter().find(|asset| &asset.asset.id == asset_id).ok_or_else(|| DatabaseError::not_found("Asset", asset_id.to_string()))
    }
}

#[async_trait]
impl Repository for MemoryAssetRepository {
    async fn enabled_assets_at_or_below(&self, rank: AssetRank) -> Result<Vec<AssetBasic>, DatabaseError> {
        self.ranks.lock().unwrap().push(rank);
        Ok(self.candidates.clone())
    }

    async fn disable_assets_with_ranks(&self, changes: Vec<RankChange>) -> Result<usize, DatabaseError> {
        let count = changes.iter().map(|change| change.asset_ids.len()).sum();
        self.changes.lock().unwrap().extend(changes);
        Ok(count)
    }

    async fn asset(&self, asset_id: AssetId) -> Result<Asset, DatabaseError> {
        Ok(self.find(&asset_id)?.asset.clone())
    }

    async fn asset_full(&self, asset_id: AssetId, _price_max_age: Duration) -> Result<AssetFull, DatabaseError> {
        let asset = self.find(&asset_id)?;
        Ok(AssetFull {
            asset: asset.asset.clone(),
            properties: asset.properties.clone(),
            score: asset.score.clone(),
            ..AssetFull::mock()
        })
    }

    async fn assets(&self, asset_ids: Vec<AssetId>) -> Result<Vec<Asset>, DatabaseError> {
        Ok(self.candidates.iter().filter(|asset| asset_ids.contains(&asset.asset.id)).map(|asset| asset.asset.clone()).collect())
    }

    async fn assets_with_prices(&self, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.price_queries.lock().unwrap().push((filters, price_max_age));
        Ok(vec![])
    }

    async fn wallet_assets_with_prices(&self, _device_id: i32, _wallet_id: i32, _since: Option<NaiveDateTime>, filters: Vec<AssetFilter>, price_max_age: Duration) -> Result<Vec<AssetPriceMetadata>, DatabaseError> {
        self.price_queries.lock().unwrap().push((filters, price_max_age));
        Ok(vec![])
    }

    async fn add_assets(&self, assets: Vec<AssetBasic>) -> Result<usize, DatabaseError> {
        let count = assets.len();
        self.added.lock().unwrap().extend(assets);
        Ok(count)
    }

    async fn update_assets(&self, asset_ids: Vec<AssetId>, updates: Vec<AssetUpdate>) -> Result<usize, DatabaseError> {
        let count = asset_ids.len();
        self.updates.lock().unwrap().push((asset_ids, updates));
        Ok(count)
    }

    async fn upsert_asset_associations(&self, _id: String, associations: Vec<AssetAssociation>) -> Result<usize, DatabaseError> {
        Ok(associations.len())
    }

    async fn add_scan_addresses(&self, addresses: Vec<ScanAddress>) -> Result<usize, DatabaseError> {
        Ok(addresses.len())
    }

    async fn update_coin_address(&self, _chain_address: ChainAddress, _balance: AssetBalance) -> Result<(), DatabaseError> {
        Ok(())
    }

    async fn update_token_addresses(&self, _chain_address: ChainAddress, _balances: Vec<AssetBalance>) -> Result<TokenAddressesUpdate, DatabaseError> {
        Ok(TokenAddressesUpdate { added: 0, unknown_asset_ids: vec![] })
    }

    async fn update_image_flags(&self, _chain: Chain, _asset_ids: HashSet<AssetId>) -> Result<(usize, usize), DatabaseError> {
        Ok((0, 0))
    }

    async fn update_price_flags(&self) -> Result<(usize, usize), DatabaseError> {
        Ok((0, 0))
    }

    async fn update_usage_ranks(&self, _windows: Vec<(NaiveDateTime, i64)>, _retain_since: NaiveDateTime, _batch_size: usize) -> Result<usize, DatabaseError> {
        Ok(0)
    }

    async fn update_perpetuals(&self, _assets: Vec<Asset>, _asset_updates: Vec<AssetUpdate>, perpetuals: Vec<Perpetual>) -> Result<Result<usize, DatabaseError>, DatabaseError> {
        Ok(Ok(perpetuals.len()))
    }

    async fn asset_catalog(&self) -> Result<AssetCatalog, DatabaseError> {
        self.catalog_reads.fetch_add(1, Ordering::Relaxed);
        Ok(self.catalog.clone())
    }

    async fn list_tag(&self, tag_id: String) -> Result<Option<Tag>, DatabaseError> {
        Ok(self.tags.iter().find(|tag| tag.id == tag_id).cloned())
    }

    async fn list_tags(&self) -> Result<Vec<Tag>, DatabaseError> {
        Ok(self.tags.clone())
    }

    async fn set_list_assets(&self, tag_id: String, _list_name: String, _list_id: ListId, asset_ids: Vec<AssetId>, is_new_tag: bool) -> Result<Option<usize>, DatabaseError> {
        let count = asset_ids.len();
        self.list_assets.lock().unwrap().push(ListAssets { tag_id, asset_ids, is_new_tag });
        Ok(Some(count))
    }
}
