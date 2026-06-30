use primitives::AssetVecExt;
use std::collections::HashSet;
use std::error::Error;

use crate::model::AssetAddressChanges;
use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use settings_chain::ChainProviders;
use storage::{AssetsAddressesRepository, AssetsRepository, Database};
use streamer::{ChainAddressPayload, StreamProducer, StreamProducerQueue, consumer::MessageConsumer};

pub struct FetchTokenAddressesConsumer {
    pub provider: ChainProviders,
    pub database: Database,
    pub stream_producer: StreamProducer,
    pub cacher: CacherClient,
}

impl FetchTokenAddressesConsumer {
    pub fn new(provider: ChainProviders, database: Database, stream_producer: StreamProducer, cacher: CacherClient) -> Self {
        Self {
            provider,
            database,
            stream_producer,
            cacher,
        }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchTokenAddressesConsumer {
    async fn should_process(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.cacher
            .can_process_cached(CacheKey::FetchTokenAddresses(payload.value.chain.as_ref(), &payload.value.address))
            .await
    }

    async fn process(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain_address = payload.value;
        let all_assets = self.provider.get_balance_assets(chain_address.chain, chain_address.address.clone()).await?;
        let existing_addresses = self.database.assets_addresses()?.get_asset_addresses(chain_address.clone())?;
        let changes = AssetAddressChanges::from_token_balances(&chain_address, existing_addresses, all_assets);

        let asset_ids: Vec<_> = changes.addresses_to_add.iter().map(|address| address.asset_id.clone()).collect();
        let existing_ids: HashSet<_> = self.database.assets()?.get_assets(asset_ids)?.ids().into_iter().collect();
        let mut addresses_to_add = Vec::new();
        let mut missing_ids = Vec::new();

        for address in changes.addresses_to_add {
            if existing_ids.contains(&address.asset_id) {
                addresses_to_add.push(address);
            } else {
                missing_ids.push(address.asset_id);
            }
        }

        let latest_count = addresses_to_add.len();
        self.database.assets_addresses()?.delete_assets_addresses(changes.addresses_to_delete)?;
        self.database.assets_addresses()?.add_assets_addresses(addresses_to_add)?;

        self.stream_producer.publish_fetch_assets(missing_ids).await?;

        Ok(latest_count)
    }
}
