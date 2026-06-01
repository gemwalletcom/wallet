use std::{error::Error, sync::Arc};

use ::nft::NFTClient;
use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use streamer::{FetchNFTAssetPayload, consumer::MessageConsumer};
use tokio::sync::Mutex;

pub struct FetchNftAssetConsumer {
    pub nft_client: Arc<Mutex<NFTClient>>,
    pub cacher: CacherClient,
}

#[async_trait]
impl MessageConsumer<FetchNFTAssetPayload, usize> for FetchNftAssetConsumer {
    async fn should_process(&self, payload: FetchNFTAssetPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let asset_id = payload.asset_id.to_string();
        self.cacher.can_process_cached(CacheKey::FetchNftAsset(&asset_id)).await
    }

    async fn process(&self, payload: FetchNFTAssetPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.nft_client.lock().await.refresh_asset(payload.asset_id).await?;
        Ok(1)
    }
}
