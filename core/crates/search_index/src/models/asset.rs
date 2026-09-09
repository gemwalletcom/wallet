use primitives::{Asset, AssetMarket, AssetProperties, AssetScore, AssetType, Chain, ChainAsset};
use serde::{Deserialize, Serialize, Serializer};

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
    #[serde(serialize_with = "AssetDocument::serialize_asset")]
    pub asset: Asset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    pub properties: AssetProperties,
    pub score: AssetScore,
    pub usage_rank: i32,
    pub market: Option<AssetMarket>,
    pub tags: Option<Vec<String>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchAsset<'a> {
    #[serde(flatten)]
    asset: &'a Asset,
    chain: Chain,
    token_id: Option<&'a str>,
}

impl AssetDocument {
    fn serialize_asset<S: Serializer>(asset: &Asset, serializer: S) -> Result<S::Ok, S::Error> {
        SearchAsset {
            asset,
            chain: asset.chain(),
            token_id: asset.token_id(),
        }
        .serialize(serializer)
    }

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
    use primitives::AssetId;
    use serde_json::Value;

    #[test]
    fn test_asset_document_serialization_preserves_search_fields() {
        let assets = [
            r#"{"id":"smartchain_0xbe9D156892E55e7154BcD3cB0FEA677F9D3103E1","name":"SpaceX","symbol":"SPCXB","decimals":18,"type":"BEP20","chain":"smartchain","tokenId":"0xbe9D156892E55e7154BcD3cB0FEA677F9D3103E1"}"#,
            r#"{"id":"ethereum","name":"Ethereum","symbol":"ETH","decimals":18,"type":"NATIVE","chain":"ethereum","tokenId":null}"#,
        ];

        for expected in assets {
            let asset: Asset = serde_json::from_str(expected).unwrap();
            let document = AssetDocument {
                id: asset.id.to_string(),
                aliases: AssetDocument::aliases(&asset),
                properties: AssetProperties::default(asset.id.clone()),
                asset: asset.clone(),
                score: AssetScore::default(),
                usage_rank: 0,
                market: None,
                tags: Some(vec!["bstocks".to_string()]),
            };

            let value = serde_json::to_value(&document).unwrap();

            assert_eq!(value["asset"], serde_json::from_str::<Value>(expected).unwrap());
            assert_eq!(serde_json::from_value::<AssetDocument>(value).unwrap().asset, asset);
            assert_eq!(serde_json::to_value(&asset).unwrap().get("chain"), None);
            assert_eq!(serde_json::to_value(&asset).unwrap().get("tokenId"), None);
        }
    }

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
