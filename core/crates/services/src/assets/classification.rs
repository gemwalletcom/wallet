use crate::ConfigCacher;
use config_keys::ConfigKey;
use primitives::{AssetBasic, AssetScore, asset_score::AssetRank};
use serde::Deserialize;
use storage::DatabaseError;

#[derive(Clone, Deserialize)]
struct FraudulentAssetRule {
    name: String,
    symbols: Vec<String>,
}

#[derive(Clone)]
pub struct AssetClassificationRules {
    spam_markers: Vec<String>,
    fraudulent_assets: Vec<FraudulentAssetRule>,
}

impl AssetClassificationRules {
    pub async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        let spam_markers = config
            .get_json::<Vec<String>>(ConfigKey::AssetsSpamMarkers)
            .await?
            .into_iter()
            .map(|marker| marker.trim().to_ascii_lowercase())
            .filter(|marker| !marker.is_empty())
            .collect();
        Ok(Self {
            spam_markers,
            fraudulent_assets: config.get_json(ConfigKey::AssetsFraudulentAssets).await?,
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
        if is_address(name) || is_address(symbol) || self.fraudulent_assets.iter().any(|rule| rule.name == name && rule.symbols.iter().any(|candidate| candidate == symbol)) {
            return Some(AssetRank::Fraudulent);
        }
        None
    }

    pub fn classified(&self, mut asset: AssetBasic) -> AssetBasic {
        if let Some(risk) = self.classify(asset.score.rank, &asset.asset.name, &asset.asset.symbol) {
            asset.score = AssetScore::new(risk.threshold());
            asset.properties.is_enabled = false;
        }
        asset
    }

    #[cfg(test)]
    pub(crate) fn mock_with_spam_marker(marker: &str) -> Self {
        Self {
            spam_markers: vec![marker.to_string()],
            fraudulent_assets: vec![],
        }
    }
}

fn is_address(value: &str) -> bool {
    let value = value.trim();
    gem_evm::validate_address(value) || gem_solana::validate_address(value) || gem_tron::validate_address(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Asset;
    use primitives::testkit::signer_mock::{TEST_EVM_SENDER, TEST_SOLANA_SENDER, TEST_TRON_SENDER};

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
        assert_eq!(rules.classify(15, TEST_EVM_SENDER, TEST_EVM_SENDER), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(15, "Tether", &TEST_EVM_SENDER.to_lowercase()), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(15, TEST_SOLANA_SENDER, "USDC"), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(15, TEST_TRON_SENDER, "USDT"), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(15, "Sui", "0x2::sui::SUI"), None);
        assert_eq!(rules.classify(15, "0x Protocol Token", "ZRX"), None);
        assert_eq!(rules.classify(15, "0x0", "ZERO"), None);
        assert_eq!(rules.classify(15, "SuperMegaUltraHyperDogeCatFrogCoin", "SMUH"), None);
        assert_eq!(rules.classify(15, "BabyDogeCoinBaseChainV2TokenMoonx", "BDC"), None);
        assert_eq!(rules.classify(15, &format!(" {TEST_TRON_SENDER} "), "USDT"), Some(AssetRank::Fraudulent));
        assert_eq!(rules.classify(25, TEST_EVM_SENDER, "USDT"), None);

        let mut asset = Asset::mock_erc20().as_basic_primitive();
        asset.asset.name = "www.example.com".to_string();

        let asset = rules.classified(asset);

        assert_eq!(asset.score.rank, AssetRank::Spam.threshold());
        assert!(!asset.properties.is_enabled);
    }
}
