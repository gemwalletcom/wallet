use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{AssetBasic, asset_score::AssetRank};
use storage::DatabaseError;

use crate::assets::repository::{RankChange, Repository};

pub(crate) struct MemoryAssetRepository {
    candidates: Vec<AssetBasic>,
    ranks: Mutex<Vec<AssetRank>>,
    changes: Mutex<Vec<RankChange>>,
}

impl MemoryAssetRepository {
    pub(crate) fn new(candidates: Vec<AssetBasic>) -> Self {
        Self {
            candidates,
            ranks: Mutex::new(Vec::new()),
            changes: Mutex::new(Vec::new()),
        }
    }

    pub(crate) fn ranks(&self) -> Vec<AssetRank> {
        self.ranks.lock().unwrap().clone()
    }

    pub(crate) fn changes(&self) -> Vec<RankChange> {
        self.changes.lock().unwrap().clone()
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
}
