use std::sync::Arc;

use primitives::{NFTAssetData, NFTAssetId, ReportNft, WalletType};

use super::model::GemCollectibleDetails;
use super::{GemNftService, rules};
use crate::services::avatar::GemAvatarService;
use crate::services::error::GemServiceError;
use crate::services::explorer::GemExplorerService;

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

    pub fn details(&self, wallet_type: WalletType, asset_data: NFTAssetData, is_owned: bool) -> GemCollectibleDetails {
        let chain = asset_data.asset.chain;
        let contract = asset_data.collection.contract_address.clone();
        let (contract_explorer, token_explorer) = if contract.is_empty() {
            (None, None)
        } else {
            (
                self.explorer.get_token_url(chain, contract.clone()),
                self.explorer.get_nft_url(chain, contract, asset_data.asset.token_id.clone()),
            )
        };
        rules::collectible_details(&wallet_type, &asset_data, is_owned, contract_explorer, token_explorer)
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
