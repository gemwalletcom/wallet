use std::collections::HashMap;
use std::error::Error;

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

use super::NFTClient;

pub struct FetchNftAssetsAddressesConsumer {
    pub cacher: CacherClient,
    pub nft_client: NFTClient,
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_process(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher.can_process_cached(CacheKey::FetchNftAssetsAddresses(payload.value.chain.as_ref(), &payload.value.address)).await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let map = HashMap::from([(payload.value.chain, payload.value.address.clone())]);
        let assets = self.nft_client.update_assets_for_addresses(map).await?;
        Ok(assets.len())
    }
}
