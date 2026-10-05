use primitives::{Asset, AssetBasic, AssetType, Chain, ChainAsset};

use super::rules::default_asset_basic;
use crate::models::asset::chain_asset_wrapper;

#[derive(Default, uniffi::Object)]
pub struct GemAssetConfigService {}

#[uniffi::export]
impl GemAssetConfigService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn default_asset_basic(&self, asset: Asset) -> AssetBasic {
        default_asset_basic(asset)
    }

    pub fn default_asset(&self, chain: Chain, asset_type: AssetType) -> Option<Asset> {
        super::rules::default_asset(chain, asset_type)
    }

    pub fn chain_asset(&self, chain: Chain) -> ChainAsset {
        chain_asset_wrapper(chain)
    }
}

impl GemAssetConfigService {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_asset_picks_the_chain_asset_of_that_type() {
        let config = GemAssetConfigService::new();
        let perpetual = config.default_asset(Chain::HyperCore, AssetType::PERPETUAL);

        assert_eq!(perpetual.map(|asset| asset.symbol), Some("USDC".to_string()));
        assert_eq!(config.default_asset(Chain::Bitcoin, AssetType::PERPETUAL), None);
    }
}
