use primitives::{Asset, AssetMarket, AssetProperties, AssetScore, AssetType, ChainAsset};
use serde::{Deserialize, Serialize};

pub const ASSETS_INDEX_NAME: &str = "assets";
pub const ASSETS_FILTERS: &[&str] = &[
    "asset.chain",
    "asset.tokenId",
    "asset.name",
    "asset.symbol",
    "asset.type",
    "score.rank",
    "properties.isEnabled",
    "properties.hasImage",
    "properties.isBuyable",
    "properties.isSellable",
    "properties.isSwapable",
    "properties.isStakeable",
    "market.marketCap",
    "market.marketCapFdv",
    "market.marketCapRank",
    "market.totalVolume",
    "tags",
];
pub const ASSETS_SEARCH_ATTRIBUTES: &[&str] = &["asset.tokenId", "asset.chain", "asset.name", "asset.symbol", "asset.type", "aliases"];
pub const ASSETS_RANKING_RULES: &[&str] = &[
    "words",
    "typo",
    "score.rank:desc",
    "properties.hasImage:desc",
    "properties.isBuyable:desc",
    "properties.isSellable:desc",
    "properties.isSwapable:desc",
    "properties.isStakeable:desc",
    "usageRank:desc",
    "market.marketCapFdv:desc",
    "proximity",
    "market.marketCapRank:asc",
    "market.marketCap:desc",
    "market.totalVolume:desc",
    "attribute",
    "exactness",
];

pub const ASSETS_SORTS: &[&str] = &["score.rank"];

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AssetDocument {
    pub id: String,
    pub asset: Asset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    pub properties: AssetProperties,
    pub score: AssetScore,
    pub usage_rank: i32,
    pub market: Option<AssetMarket>,
    pub tags: Option<Vec<String>>,
}

impl AssetDocument {
    pub fn aliases(asset: &Asset) -> Option<Vec<String>> {
        if asset.asset_type != AssetType::NATIVE {
            return None;
        }

        let network_name = ChainAsset::from_chain(asset.chain()).network_name;
        Some(vec![asset.chain().to_string(), network_name])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain};

    #[test]
    fn native_asset_aliases_include_chain_id_and_network_name() {
        let ton = Asset::from_chain(Chain::Ton);
        let base = Asset::from_chain(Chain::Base);

        assert_eq!(AssetDocument::aliases(&ton), Some(vec!["ton".to_string(), "TON".to_string()]));
        assert_eq!(AssetDocument::aliases(&base), Some(vec!["base".to_string(), "Base".to_string()]));
    }

    #[test]
    fn token_aliases_are_empty() {
        let token = Asset::new(AssetId::token(Chain::Ton, "jetton"), "Token".to_string(), "TOKEN".to_string(), 9, AssetType::JETTON);

        assert_eq!(AssetDocument::aliases(&token), None);
    }

    #[test]
    fn asset_chain_is_searchable_for_compound_queries() {
        assert!(ASSETS_SEARCH_ATTRIBUTES.contains(&"asset.chain"));
        assert!(ASSETS_SEARCH_ATTRIBUTES.contains(&"aliases"));
    }

    #[test]
    fn asset_tags_are_filterable_not_searchable() {
        assert!(ASSETS_FILTERS.contains(&"tags"));
        assert!(!ASSETS_SEARCH_ATTRIBUTES.contains(&"tags"));
    }
}
