use std::sync::Arc;

use primitives::{Chain, NFTAssetId, ReportNft, Wallet};

use super::{GemNftService, rules};
use crate::services::avatar::GemAvatarService;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCollectibleLinks {
    pub contract: Option<BlockExplorerLink>,
    pub token: Option<BlockExplorerLink>,
}

#[derive(uniffi::Object)]
pub struct GemCollectibleService {
    nfts: Arc<GemNftService>,
    avatars: Arc<GemAvatarService>,
    explorer: Arc<GemExplorerService>,
}

#[uniffi::export]
impl GemCollectibleService {
    #[uniffi::constructor]
    pub fn new(nfts: Arc<GemNftService>, avatars: Arc<GemAvatarService>, explorer: Arc<GemExplorerService>) -> Self {
        Self { nfts, avatars, explorer }
    }

    pub fn can_send(&self, wallet: Wallet, chain: Chain) -> bool {
        rules::can_send(&wallet.wallet_type, chain)
    }

    pub fn links(&self, chain: Chain, contract_address: String, token_id: String) -> GemCollectibleLinks {
        GemCollectibleLinks {
            contract: self.explorer.get_token_url(chain, contract_address.clone()),
            token: self.explorer.get_nft_url(chain, contract_address, token_id),
        }
    }

    pub async fn refresh_asset(&self, asset_id: NFTAssetId) -> Result<(), GemServiceError> {
        self.nfts.refresh_asset(self.nfts.session.current_wallet_id()?, asset_id).await
    }

    pub async fn report(&self, report: ReportNft) -> Result<(), GemServiceError> {
        self.nfts.report(report).await
    }

    pub async fn set_wallet_avatar(&self, url: String) -> Result<(), GemServiceError> {
        self.avatars.set_image_url(self.nfts.session.current_wallet_id()?, url).await
    }
}
