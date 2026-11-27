use std::collections::HashMap;
use std::error::Error;

use cacher::{CacherClient, ThrottledTask};
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

use super::NFTClient;

pub struct FetchNftAssetsAddressesConsumer {
    pub throttle: CacherClient,
    pub nft_client: NFTClient,
}

impl MessageConsumer<ChainAddressPayload, usize> for FetchNftAssetsAddressesConsumer {
    async fn should_consume(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledTask::FetchNftAssetsAddresses {
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
