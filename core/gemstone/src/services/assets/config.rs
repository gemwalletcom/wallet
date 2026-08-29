use primitives::{Asset, AssetId, Chain, ChainAsset, WalletType};

use super::model::{GemAssetAction, GemAssetFilter};
use super::rules::{asset_action_filters, default_token_chain, popular_asset_ids};
use crate::models::asset::{
    asset_default_rank, asset_ids_enabled_by_default, asset_is_swapable, chain_asset_wrapper, chain_fee_asset_ids, default_token_rank, wallet_asset_is_enabled,
    wallet_default_assets,
};
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

    pub fn chain_fee_asset_ids(&self, chain: Chain) -> Vec<AssetId> {
        chain_fee_asset_ids(chain)
    }

    pub fn chain_asset(&self, chain: Chain) -> ChainAsset {
        chain_asset_wrapper(chain)
    }

    pub fn ids_enabled_by_default(&self) -> Vec<AssetId> {
        asset_ids_enabled_by_default()
    }

    pub fn is_enabled(&self, asset_id: AssetId, wallet_type: WalletType) -> bool {
        wallet_asset_is_enabled(asset_id, wallet_type)
    }

    pub fn is_swapable(&self, asset_id: AssetId) -> bool {
        asset_is_swapable(asset_id)
    }

    pub fn action_filters(&self, action: GemAssetAction) -> Vec<GemAssetFilter> {
        asset_action_filters(action)
    }

    pub fn popular_ids(&self) -> Vec<AssetId> {
        popular_asset_ids()
    }

    pub fn default_token_chain(&self, chains: Vec<Chain>) -> Option<Chain> {
        default_token_chain(&chains)
    }

    pub fn matching_assets(&self, assets: Vec<Asset>, query: String) -> Vec<Asset> {
        matching_assets(assets, &query)
    }
}
