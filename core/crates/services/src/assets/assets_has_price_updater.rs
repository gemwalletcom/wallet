use primitives::AssetId;
use std::collections::HashSet;
use std::error::Error;
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database, DatabaseError, PricesRepository};

pub struct AssetsHasPriceUpdater {
    database: Database,
}

impl AssetsHasPriceUpdater {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn update(&self) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        Ok(self
            .database
            .run(|client| -> Result<(usize, usize), DatabaseError> {
                let eligible: HashSet<AssetId> = client.get_prices_asset_ids()?.into_iter().collect();

                let current: HashSet<AssetId> = client.get_asset_ids_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::HasPrice(true)])?.into_iter().collect();

                let additions: Vec<AssetId> = eligible.difference(&current).cloned().collect();
                let removals: Vec<AssetId> = current.difference(&eligible).cloned().collect();

                let additions_len = additions.len();
                let removals_len = removals.len();

                if !additions.is_empty() {
                    client.update_assets(additions, vec![AssetUpdate::HasPrice(true)])?;
                }
                if !removals.is_empty() {
                    client.update_assets(removals, vec![AssetUpdate::HasPrice(false)])?;
                }

                Ok((additions_len, removals_len))
            })
            .await?)
    }
}
