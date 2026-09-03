use primitives::{Asset, AssetId, Chain, ChainAsset};

use super::rules::{default_token_chain, icon_asset_id, popular_asset_ids};
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

    pub fn default_rank(&self, asset_id: AssetId) -> i32 {
        asset_default_rank(asset_id)
    }

    pub fn default_token_rank(&self) -> i32 {
        default_token_rank()
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

    pub fn icon_asset_id(&self, asset_id: AssetId) -> AssetId {
        icon_asset_id(&asset_id)
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
