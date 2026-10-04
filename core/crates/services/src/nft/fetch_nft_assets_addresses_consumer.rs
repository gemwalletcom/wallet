use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

use super::NFTClient;
use crate::fetch_throttle::{FetchThrottle, ThrottledFetch};

pub struct FetchNftAssetsAddressesConsumer {
    pub throttle: Arc<dyn FetchThrottle>,
    pub nft_client: NFTClient,
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_consume(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledFetch::NftAssetsAddresses {
                chain: payload.value.chain.as_ref(),
                address: &payload.value.address,
            })
            .await
    }

    async fn consume(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let map = HashMap::from([(payload.value.chain, payload.value.address.clone())]);
        let assets = self.nft_client.update_assets_for_addresses(map).await?;
        Ok(assets.len())
    }
}
