use std::{collections::HashMap, sync::LazyLock};

use primitives::{AssetBasic, AssetScore, asset_score::AssetRank};

const SPAM_MARKERS: &[&str] = &["www."];
static SUSPICIOUS_ASSETS: LazyLock<HashMap<&'static str, &'static [&'static str]>> = LazyLock::new(|| {
    let mut assets = HashMap::new();
    assets.insert("Tether", &["USDT"][..]);
    assets.insert("Tether USD", &["USDT", "$USD₮"][..]);
    assets.insert("USD Coin", &["USDC"][..]);
    assets
});

pub(crate) fn classify(rank: i32, name: &str, symbol: &str) -> Option<AssetRank> {
    if rank > AssetRank::Trivial.threshold() {
        return None;
    }
    let metadata = [name.to_ascii_lowercase(), symbol.to_ascii_lowercase()];
    if SPAM_MARKERS.iter().any(|marker| metadata.iter().any(|value| value.contains(marker))) {
        return Some(AssetRank::Spam);
    }
    if SUSPICIOUS_ASSETS.get(name).is_some_and(|symbols| symbols.contains(&symbol)) {
        return Some(AssetRank::Fraudulent);
    }
    None
}

pub(crate) fn apply(mut asset: AssetBasic) -> AssetBasic {
    if let Some(risk) = classify(asset.score.rank, &asset.asset.name, &asset.asset.symbol) {
        asset.score = AssetScore::new(risk.threshold());
        asset.properties.is_enabled = false;
    }
    asset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        assert_eq!(classify(15, "www.example.com", "URL"), Some(AssetRank::Spam));
        assert_eq!(classify(15, "Example", "WWW.EXAMPLE.COM"), Some(AssetRank::Spam));
        assert_eq!(classify(15, "Tether", "USDT"), Some(AssetRank::Fraudulent));
        assert_eq!(classify(25, "www.example.com", "URL"), None);
        assert_eq!(classify(15, "Bitcoin", "BTC"), None);
    }

    #[test]
    fn test_apply() {
        let mut asset = primitives::Asset::mock_erc20().as_basic_primitive();
        asset.asset.name = "www.example.com".to_string();

        let asset = apply(asset);

        assert_eq!(asset.score.rank, AssetRank::Spam.threshold());
        assert!(!asset.properties.is_enabled);
    }
}
