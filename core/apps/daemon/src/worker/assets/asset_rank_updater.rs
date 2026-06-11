use primitives::{AssetId, asset_score::AssetRank};
use std::{collections::HashMap, error::Error, sync::LazyLock};
use storage::{AssetFilter, AssetUpdate, AssetsRepository, Database};

static SUSPICIOUS_ASSETS: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {
    let mut assets = HashMap::new();
    assets.insert("Tether", &["USDT"][..]);
    assets.insert("Tether USD", &["USDT", "$USD₮"][..]);
    assets.insert("USD Coin", &["USDC"][..]);
    assets
});

pub struct AssetRankUpdater {
    database: Database,
}

impl AssetRankUpdater {
    pub fn new(database: Database) -> Self {
        AssetRankUpdater { database }
    }

    pub async fn update_suspicious_assets(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let assets = self.database.assets()?.get_assets_by_filter(vec![AssetFilter::IsEnabled(true), AssetFilter::RankLte(15)])?;
        let asset_ids: Vec<AssetId> = assets
            .into_iter()
            .filter(|x| is_suspicious(x.score.rank, &x.asset.name, &x.asset.symbol))
            .map(|x| x.asset.id)
            .collect();

        let updates = vec![AssetUpdate::Rank(AssetRank::Fraudulent.threshold()), AssetUpdate::IsEnabled(false)];
        Ok(self.database.assets()?.update_assets(asset_ids, updates)?)
    }
}

fn is_suspicious(rank: i32, name: &str, symbol: &str) -> bool {
    rank <= 15 && SUSPICIOUS_ASSETS.get(name).is_some_and(|symbols| symbols.contains(&symbol))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_suspicious() {
        assert!(is_suspicious(10, "Tether", "USDT"));

        assert!(!is_suspicious(25, "Tether", "USDT"));
        assert!(!is_suspicious(10, "Bitcoin", "BTC"));
    }
}
