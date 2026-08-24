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
    pub(crate) fn supports_ingress(&self) -> bool {
        self.enabled && matches!(self.direction, AssetDirection::Both | AssetDirection::Ingress)
    }

    pub(crate) fn supports_egress(&self) -> bool {
        self.enabled && matches!(self.direction, AssetDirection::Both | AssetDirection::Egress)
    }
}

impl AssetsResponse {
    pub(crate) fn asset(&self, asset: &ChainflipAsset) -> Option<&BrokerAsset> {
        self.assets
            .iter()
            .find(|broker_asset| broker_asset.network == asset.chain && broker_asset.ticker == asset.asset)
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
    fn test_asset_swap_directions() {
        let source = asset("Ethereum", "ETH");
        let destination = asset("Tron", "TRX");

        let assets = response();
        let source_asset = assets.asset(&source).unwrap();
        let destination_asset = assets.asset(&destination).unwrap();
        assert_eq!(source_asset.id, "eth.eth");
        assert_eq!(source_asset.minimal_amount_native, BigUint::from(10_000_000_000_000_000u64));
        assert_eq!(destination_asset.id, "trx.tron");
        assert!(source_asset.supports_ingress());
        assert!(destination_asset.supports_egress());

        let mut assets = response();
        assets.assets[0].enabled = false;
        assert!(!assets.assets[0].supports_ingress());

        let mut assets = response();
        assets.assets[1].enabled = false;
        assert!(!assets.assets[1].supports_egress());

        let mut assets = response();
        assets.assets[0].direction = AssetDirection::Egress;
        assert!(!assets.assets[0].supports_ingress());

        let mut assets = response();
        assets.assets[1].direction = AssetDirection::Ingress;
        assert!(!assets.assets[1].supports_egress());
    }
}
