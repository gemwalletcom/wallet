use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use chrono::{Duration, NaiveDateTime, Utc};
use primitives::AssetId;

use crate::assets::repository::Repository;

const RETENTION_DAYS: i64 = 30;

#[derive(Clone, Copy)]
pub struct UsageRankUpdaterConfig {
    pub batch_size: usize,
}

pub struct UsageRankUpdater {
    repository: Arc<dyn Repository>,
    config: UsageRankUpdaterConfig,
}

impl UsageRankUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>, config: UsageRankUpdaterConfig) -> Self {
        UsageRankUpdater { repository, config }
    }

    pub async fn update_usage_ranks(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let now = Utc::now().naive_utc();
        let retain_since = now - Duration::days(RETENTION_DAYS);
        Ok(self.repository.update_usage_ranks(usage_windows(now), retain_since, self.config.batch_size).await?)
    }
}

fn usage_windows(now: NaiveDateTime) -> Vec<(NaiveDateTime, i64)> {
    vec![(now - Duration::hours(1), 250), (now - Duration::days(1), 100), (now - Duration::days(7), 10), (now - Duration::days(RETENTION_DAYS), 1)]
}

pub(crate) fn usage_ranks(weighted_counts: Vec<(Vec<(AssetId, i64)>, i64)>) -> Vec<(AssetId, i32)> {
    let mut raw_scores: HashMap<AssetId, i64> = HashMap::new();
    for (counts, weight) in weighted_counts {
        for (asset_id, count) in counts {
            *raw_scores.entry(asset_id).or_insert(0) += count * weight;
        }
    }
    usage_ranks_from_scores(raw_scores)
}

#[cfg(test)]
fn calculate_usage_ranks(counts_1h: &[(AssetId, i64)], counts_24h: &[(AssetId, i64)], counts_7d: &[(AssetId, i64)], counts_30d: &[(AssetId, i64)]) -> Vec<(AssetId, i32)> {
    let counts = [counts_1h, counts_24h, counts_7d, counts_30d].map(<[(AssetId, i64)]>::to_vec);
    usage_ranks(counts.into_iter().zip(usage_windows(Utc::now().naive_utc())).map(|(counts, (_, weight))| (counts, weight)).collect())
}

fn usage_ranks_from_scores(raw_scores: HashMap<AssetId, i64>) -> Vec<(AssetId, i32)> {
    if raw_scores.is_empty() {
        return vec![];
    }

    let mut scores: Vec<(AssetId, i64)> = raw_scores.into_iter().collect();
    scores.sort_by_key(|a| a.1);

    let total = scores.len() as f64;
    scores
        .into_iter()
        .enumerate()
        .map(|(position, (asset_id, _))| {
            let percentile = ((position as f64 + 1.0) / total * 100.0).round() as i32;
            (asset_id, percentile.min(100))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_calculate_usage_ranks_empty() {
        let result = calculate_usage_ranks(&[], &[], &[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_calculate_usage_ranks_single() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 10)];
        let result = calculate_usage_ranks(&counts_1h, &[], &[], &[]);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, Chain::Bitcoin.as_asset_id());
        assert_eq!(result[0].1, 100);
    }

    #[test]
    fn test_calculate_usage_ranks_multiple() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 100), (Chain::Ethereum.as_asset_id(), 10), (Chain::Solana.as_asset_id(), 1)];
        let result = calculate_usage_ranks(&counts_1h, &[], &[], &[]);
        assert_eq!(result.len(), 3);

        let asset1_rank = result.iter().find(|(id, _)| *id == Chain::Bitcoin.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset2_rank = result.iter().find(|(id, _)| *id == Chain::Ethereum.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset3_rank = result.iter().find(|(id, _)| *id == Chain::Solana.as_asset_id()).map(|(_, r)| *r).unwrap();

        assert_eq!(asset3_rank, 33);
        assert_eq!(asset2_rank, 67);
        assert_eq!(asset1_rank, 100);
    }

    #[test]
    fn test_calculate_usage_ranks_weighted() {
        let counts_1h = vec![(Chain::Bitcoin.as_asset_id(), 2)];
        let counts_24h = vec![(Chain::Ethereum.as_asset_id(), 25)];
        let counts_7d = vec![(Chain::Solana.as_asset_id(), 300)];
        let counts_30d = vec![(Chain::SmartChain.as_asset_id(), 4000)];
        let result = calculate_usage_ranks(&counts_1h, &counts_24h, &counts_7d, &counts_30d);

        let asset1_rank = result.iter().find(|(id, _)| *id == Chain::Bitcoin.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset2_rank = result.iter().find(|(id, _)| *id == Chain::Ethereum.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset3_rank = result.iter().find(|(id, _)| *id == Chain::Solana.as_asset_id()).map(|(_, r)| *r).unwrap();
        let asset4_rank = result.iter().find(|(id, _)| *id == Chain::SmartChain.as_asset_id()).map(|(_, r)| *r).unwrap();

        assert_eq!(asset1_rank, 25);
        assert_eq!(asset2_rank, 50);
        assert_eq!(asset3_rank, 75);
        assert_eq!(asset4_rank, 100);
    }
}
