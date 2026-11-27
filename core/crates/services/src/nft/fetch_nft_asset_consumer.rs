use std::error::Error;

use crate::nft::NFTClient;
use cacher::{CacherClient, ThrottledTask};
use streamer::{FetchNFTAssetPayload, consumer::MessageConsumer};

pub struct FetchNftAssetConsumer {
    pub nft_client: NFTClient,
    pub throttle: CacherClient,
}

impl MessageConsumer<FetchNFTAssetPayload, usize> for FetchNftAssetConsumer {
    async fn should_consume(&self, payload: &FetchNFTAssetPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let asset_id = payload.asset_id.to_string();
        self.throttle.try_start(ThrottledTask::FetchNftAsset { asset_id: &asset_id }).await
    }

    async fn consume(&self, payload: FetchNFTAssetPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.nft_client.refresh_asset(payload.asset_id).await?;
        Ok(1)
    }
}
