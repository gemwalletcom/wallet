use primitives::{Asset, AssetBasic, AssetId, AssetType, Chain, ChainAsset};

use super::model::GemAssetSectionIds;
use super::rules::{asset_sections, default_asset_basic, popular_asset_ids};
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

    pub fn asset_sections(&self, ids: Vec<AssetId>, pinned_ids: Vec<AssetId>, shows_popular: bool) -> GemAssetSectionIds {
        asset_sections(ids, pinned_ids, shows_popular, popular_asset_ids())
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
