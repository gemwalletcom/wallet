use std::error::Error;
use std::sync::Arc;

use async_trait::async_trait;
use chain_providers::ChainProviders;
use storage::Database;
use streamer::{ChainAddressPayload, consumer::MessageConsumer};

use super::addresses::update_coin_address;
use crate::throttle_cacher::{ThrottleCacher, ThrottledTask};

pub struct FetchCoinAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub throttle: Arc<dyn ThrottleCacher>,
}

impl FetchCoinAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, throttle: Arc<dyn ThrottleCacher>) -> Self {
        Self { provider, database, throttle }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, String> for FetchCoinAddressesConsumer {
    async fn should_consume(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledTask::FetchCoinAddresses {
                chain: payload.value.chain.as_ref(),
                address: &payload.value.address,
            })
            .await
    }

    async fn consume(&self, payload: ChainAddressPayload) -> Result<String, Box<dyn Error + Send + Sync>> {
        let chain_address = payload.value;
        let balance = self.provider.get_balance_coin(chain_address.chain, chain_address.address.clone()).await?;
        let balance_value = balance.balance.available.to_string();
        self.database.run(move |client| update_coin_address(client, &chain_address, balance)).await?;
        Ok(balance_value)
    }
}
