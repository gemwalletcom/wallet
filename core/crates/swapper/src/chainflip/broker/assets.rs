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
    pub fn minimum_amount(&self, source_asset: &ChainflipAsset, destination_asset: &ChainflipAsset) -> Option<BigUint> {
        let source_asset = self
            .assets
            .iter()
            .find(|broker_asset| broker_asset.network == source_asset.chain && broker_asset.ticker == source_asset.asset)
            .filter(|broker_asset| broker_asset.supports_ingress())?;
        self.assets
            .iter()
            .find(|broker_asset| broker_asset.network == destination_asset.chain && broker_asset.ticker == destination_asset.asset)
            .filter(|broker_asset| broker_asset.supports_egress())?;
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
