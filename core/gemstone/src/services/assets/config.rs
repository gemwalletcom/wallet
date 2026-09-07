use primitives::{Asset, AssetBasic, AssetId, AssetType, Chain, ChainAsset};

use super::icon::{GemAssetIcon, asset_icon};
use super::rules::{default_asset_basic, default_token_chain, popular_asset_ids};
use crate::models::asset::{asset_default_rank, asset_is_swapable, chain_asset_wrapper, chain_fee_asset_ids, default_token_rank, wallet_default_assets};
use crate::services::confirm::{GemAcquireAssetFlow, acquire_asset_flow};
use crate::services::search::rules::matching_assets;

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

    pub fn default_rank(&self, asset_id: AssetId) -> i32 {
        asset_default_rank(asset_id)
    }

    pub fn default_token_rank(&self) -> i32 {
        default_token_rank()
    }

    pub fn default_asset(&self, chain: Chain, asset_type: AssetType) -> Option<Asset> {
        wallet_default_assets(chain).into_iter().find(|asset| asset.asset_type == asset_type)
    }

    pub fn wallet_default_assets(&self, chain: Chain) -> Vec<Asset> {
        wallet_default_assets(chain)
    }

    pub fn chain_asset(&self, chain: Chain) -> ChainAsset {
        chain_asset_wrapper(chain)
    }

    pub fn acquire_flow(&self, chain: Chain) -> GemAcquireAssetFlow {
        acquire_asset_flow(chain)
    }

    pub fn asset_icon(&self, asset_id: AssetId) -> GemAssetIcon {
        asset_icon(&asset_id)
    }

    pub fn is_swapable(&self, asset_id: AssetId) -> bool {
        asset_is_swapable(asset_id)
    }

    pub fn popular_ids(&self) -> Vec<AssetId> {
        popular_asset_ids()
    }

    pub fn matching_assets(&self, assets: Vec<Asset>, query: String) -> Vec<Asset> {
        matching_assets(assets, &query)
    }
}

impl GemAssetConfigService {
    pub fn chain_fee_asset_ids(&self, chain: Chain) -> Vec<AssetId> {
        chain_fee_asset_ids(chain)
    }
    pub fn default_token_chain(&self, chains: Vec<Chain>) -> Option<Chain> {
        default_token_chain(&chains)
    }
}

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
