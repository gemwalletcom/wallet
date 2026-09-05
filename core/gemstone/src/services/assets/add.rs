use std::sync::Arc;

use primitives::{Asset, AssetId, Chain, Wallet};

use super::rules;
use crate::address::checksum_address;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::chain::rules::matching_chains;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use primitives::BlockExplorerLink;

#[derive(uniffi::Object)]
pub struct GemAddAssetService {
    assets: Arc<GemAssetsService>,
    balances: Arc<GemBalanceService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemAddAssetService {
    #[uniffi::constructor]
    pub fn new(assets: Arc<GemAssetsService>, balances: Arc<GemBalanceService>, explorer: Arc<GemExplorerService>) -> Self {
        Self { assets, balances, explorer }
    }

    pub fn chains(&self, wallet: Wallet) -> Vec<Chain> {
        rules::token_chains(&wallet)
    }

    pub fn default_chain(&self, chains: Vec<Chain>) -> Option<Chain> {
        rules::default_token_chain(&chains)
    }

    pub fn matching_chains(&self, chains: Vec<Chain>, query: String) -> Vec<Chain> {
        matching_chains(chains, &query)
    }

    pub fn token_url(&self, chain: Chain, token_id: String) -> Option<BlockExplorerLink> {
        self.explorer.get_token_url(chain, token_id)
    }

    pub async fn token(&self, chain: Chain, address: String) -> Result<Asset, GemServiceError> {
        self.assets.ensure_token_asset(AssetId::from(chain, Some(checksum_address(&address, chain)))).await
    }

    pub async fn add(&self, wallet: Wallet, asset_id: AssetId) -> Result<(), GemServiceError> {
        if wallet.account(asset_id.chain).is_none() {
            return Err(GemServiceError::NotFound {
                msg: format!("wallet has no account for {}", asset_id.chain),
            });
        }
        let asset = self.assets.ensure_token_asset(asset_id).await?;
        self.balances.set_assets_enabled(wallet.id, vec![asset.id], true).await
    }
}
