use std::collections::HashSet;
use std::error::Error;

use gem_tracing::info_with_fields;
use primitives::AssetId;
use storage::{AssetsUsageRanksRepository, Database, DatabaseError, PricesRepository};
use streamer::{StreamProducer, StreamProducerQueue};

const MAX_ASSETS_PER_RUN: usize = 1;

pub struct MissingPricesPublisher {
    database: Database,
    stream_producer: StreamProducer,
}

impl MissingPricesPublisher {
    pub fn new(database: Database, stream_producer: StreamProducer) -> Self {
        Self { database, stream_producer }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let asset_ids: Vec<AssetId> = self
            .database
            .run(|client| -> Result<Vec<AssetId>, DatabaseError> {
                let ranks = client.get_all_usage_ranks()?;
                let priced: HashSet<AssetId> = client.get_prices_asset_ids()?.into_iter().collect();
                Ok(missing_assets(ranks, &priced).into_iter().take(MAX_ASSETS_PER_RUN).collect())
            })
            .await?;
        let count = asset_ids.len();
        self.stream_producer.publish_fetch_prices_assets(asset_ids.clone()).await?;
        for asset_id in &asset_ids {
            info_with_fields!("publish missing prices", asset_id = asset_id.to_string());
        }
        Ok(count)
    }
}

fn missing_assets(ranks: Vec<(AssetId, i32)>, priced: &HashSet<AssetId>) -> Vec<AssetId> {
    let mut candidates: Vec<(AssetId, i32)> = ranks.into_iter().filter(|(id, _)| !priced.contains(id)).collect();
    candidates.sort_by_key(|(_, rank)| std::cmp::Reverse(*rank));
    candidates.into_iter().map(|(id, _)| id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_missing_assets() {
        let priced = Chain::Ethereum.as_asset_id();
        let unpriced_high = Chain::Solana.as_asset_id();
        let unpriced_low = Chain::Bitcoin.as_asset_id();
        let ranks = vec![(unpriced_high.clone(), 80), (priced.clone(), 100), (unpriced_low.clone(), 20)];
        let priced: HashSet<AssetId> = [priced].into_iter().collect();

        assert_eq!(missing_assets(ranks, &priced), vec![unpriced_high, unpriced_low]);
    }
}
