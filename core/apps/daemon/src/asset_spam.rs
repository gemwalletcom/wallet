use primitives::{AssetBasic, AssetScore, ConfigKey, asset_score::AssetRank};
use serde::Deserialize;
use storage::{ConfigCacher, DatabaseError};

#[derive(Clone, Deserialize)]
struct FraudulentAssetRule {
    name: String,
    symbols: Vec<String>,
}

#[derive(Clone)]
pub(crate) struct AssetClassificationRules {
    spam_markers: Vec<String>,
    fraudulent_assets: Vec<FraudulentAssetRule>,
}

impl AssetClassificationRules {
    pub fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        let spam_markers = config
            .get_vec_string(ConfigKey::AssetsSpamMarkers)?
            .into_iter()
            .map(|marker| marker.trim().to_ascii_lowercase())
            .filter(|marker| !marker.is_empty())
            .collect();
        Ok(Self {
            spam_markers,
            fraudulent_assets: config.get_json(ConfigKey::AssetsFraudulentAssets)?,
        })
    }

    pub fn classify(&self, rank: i32, name: &str, symbol: &str) -> Option<AssetRank> {
        if rank > AssetRank::Trivial.threshold() {
            return None;
        }
        let metadata = [name.to_ascii_lowercase(), symbol.to_ascii_lowercase()];
        if self.spam_markers.iter().any(|marker| metadata.iter().any(|value| value.contains(marker))) {
            return Some(AssetRank::Spam);
        }
        if self
            .fraudulent_assets
            .iter()
            .any(|rule| rule.name == name && rule.symbols.iter().any(|candidate| candidate == symbol))
        {
            return Some(AssetRank::Fraudulent);
        }
        None
    }

    pub fn apply(&self, mut asset: AssetBasic) -> AssetBasic {
        if let Some(risk) = self.classify(asset.score.rank, &asset.asset.name, &asset.asset.symbol) {
            asset.score = AssetScore::new(risk.threshold());
            asset.properties.is_enabled = false;
        }
        asset
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Asset;

    #[test]
    fn test_asset_classification_rules() {
        let rules = AssetClassificationRules {
            spam_markers: vec!["www.".to_string()],
            fraudulent_assets: vec![FraudulentAssetRule {
                name: "Tether".to_string(),
                symbols: vec!["USDT".to_string()],
            }],
        };
        assert_eq!(rules.classify(15, "www.example.com", "URL"), Some(AssetRank::Spam));
        assert_eq!(rules.classify(15, "Example", "WWW.EXAMPLE.COM"), Some(AssetRank::Spam));
        assert_eq!(rules.classify(15, "Tether", "USDT"), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(25, "www.example.com", "URL"), None);
        assert_eq!(rules.classify(15, "Bitcoin", "BTC"), None);

        let mut asset = Asset::mock_erc20().as_basic_primitive();
        asset.asset.name = "www.example.com".to_string();

        let asset = rules.apply(asset);

        assert_eq!(asset.score.rank, AssetRank::Spam.threshold());
        assert!(!asset.properties.is_enabled);
    }
}
