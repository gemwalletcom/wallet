use std::sync::Arc;

use primitives::{NFTAssetData, NFTAssetId, NFTData, ReportNft};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemNftService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemNftService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_assets(&self, wallet_id: String) -> Result<Vec<NFTData>, GemApiError> {
        Ok(self.api.client.get_nft_assets(wallet_id).await?)
    }

    pub async fn get_asset(&self, asset_id: NFTAssetId) -> Result<NFTAssetData, GemApiError> {
        Ok(self.api.client.get_nft_asset(asset_id).await?)
    }

    pub async fn refresh_asset(&self, wallet_id: String, asset_id: NFTAssetId) -> Result<(), GemApiError> {
        self.api.client.refresh_nft_asset(wallet_id, asset_id).await?;
        Ok(())
    }

    pub async fn report(&self, report: ReportNft) -> Result<(), GemApiError> {
        self.api.client.report_nft(report).await?;
        Ok(())
    }
}
