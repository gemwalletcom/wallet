use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDateTime;
use primitives::{Asset, AssetAssociation, AssetBasic, AssetFull, AssetId, AssetPriceMetadata, ScanAddress, asset_score::AssetRank};
use storage::{AssetFilter, AssetUpdate, DatabaseError};

use crate::assets::repository::{RankChange, Repository};

pub(crate) struct MemoryAssetRepository {
    candidates: Vec<AssetBasic>,
    ranks: Mutex<Vec<AssetRank>>,
    changes: Mutex<Vec<RankChange>>,
    price_queries: Mutex<Vec<(Vec<AssetFilter>, Duration)>>,
    updates: Mutex<Vec<(Vec<AssetId>, Vec<AssetUpdate>)>>,
    added: Mutex<Vec<AssetBasic>>,
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
        }
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
}
