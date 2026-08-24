use num_bigint::BigUint;
use serde::Deserialize;
use serde_serializers::deserialize_biguint_from_str;

use super::ChainflipAsset;

#[derive(Debug, Clone, Deserialize)]
pub struct AssetsResponse {
    pub assets: Vec<BrokerAsset>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrokerAsset {
    pub id: String,
    enabled: bool,
    direction: AssetDirection,
    pub ticker: String,
    pub network: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub minimal_amount_native: BigUint,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AssetDirection {
    Both,
    Ingress,
    Egress,
    #[serde(other)]
    Unknown,
}

impl BrokerAsset {
    fn supports_ingress(&self) -> bool {
        self.enabled && matches!(self.direction, AssetDirection::Both | AssetDirection::Ingress)
    }

    fn supports_egress(&self) -> bool {
        self.enabled && matches!(self.direction, AssetDirection::Both | AssetDirection::Egress)
    }
}

impl AssetsResponse {
    fn asset(&self, asset: &ChainflipAsset) -> Option<&BrokerAsset> {
        self.assets
            .iter()
            .find(|broker_asset| broker_asset.network == asset.chain && broker_asset.ticker == asset.asset)
    }

    pub fn quote_asset_id(&self, asset: &ChainflipAsset) -> Option<&str> {
        self.asset(asset).map(|asset| asset.id.as_str())
    }

    pub fn minimum_amount(&self, source_asset: &ChainflipAsset, destination_asset: &ChainflipAsset) -> Option<BigUint> {
        let source_asset = self.asset(source_asset).filter(|broker_asset| broker_asset.supports_ingress())?;
        self.asset(destination_asset).filter(|broker_asset| broker_asset.supports_egress())?;
        Some(source_asset.minimal_amount_native.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(chain: &str, asset: &str) -> ChainflipAsset {
        ChainflipAsset {
            chain: chain.to_string(),
            asset: asset.to_string(),
        }
    }

    fn response() -> AssetsResponse {
        serde_json::from_str(include_str!("./test/assets.json")).unwrap()
    }

    #[test]
    fn test_minimum_amount_requires_enabled_swap_directions() {
        let source = asset("Ethereum", "ETH");
        let destination = asset("Tron", "TRX");

        assert_eq!(response().minimum_amount(&source, &destination), Some(BigUint::from(10_000_000_000_000_000u64)));
        assert_eq!(response().quote_asset_id(&source), Some("eth.eth"));

        let mut assets = response();
        assets.assets[0].enabled = false;
        assert!(assets.minimum_amount(&source, &destination).is_none());

        let mut assets = response();
        assets.assets[1].enabled = false;
        assert!(assets.minimum_amount(&source, &destination).is_none());

        let mut assets = response();
        assets.assets[0].direction = AssetDirection::Egress;
        assert!(assets.minimum_amount(&source, &destination).is_none());

        let mut assets = response();
        assets.assets[1].direction = AssetDirection::Ingress;
        assert!(assets.minimum_amount(&source, &destination).is_none());
    }
}
