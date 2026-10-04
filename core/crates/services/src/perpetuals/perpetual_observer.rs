use std::collections::HashSet;
use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use chain_providers::ChainProviders;
use chain_traits::TransactionsRequest;
use config_keys::ConfigParamKey;
use gem_tracing::{error_with_fields, info_with_fields};
use primitives::Chain;
use streamer::steam_producer_queue::StreamProducerQueue;
use streamer::{StreamProducer, TransactionsPayload};

use super::{PerpetualAddressCacher, PerpetualAddressTier};

pub struct PerpetualPositionObserver {
    chain: Chain,
    providers: Arc<ChainProviders>,
    addresses: Arc<dyn PerpetualAddressCacher>,
    config: Arc<ConfigCacher>,
    stream_producer: StreamProducer,
}

impl PerpetualPositionObserver {
    pub fn new(chain: Chain, providers: Arc<ChainProviders>, addresses: Arc<dyn PerpetualAddressCacher>, config: Arc<ConfigCacher>, stream_producer: StreamProducer) -> Self {
        Self {
            chain,
            providers,
            addresses,
            config,
            stream_producer,
        }
    }

    pub async fn observe_active(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let active = self.addresses.addresses(self.chain, PerpetualAddressTier::Active).await?;
        let priority = self.addresses.addresses(self.chain, PerpetualAddressTier::Priority).await?;
        let excluded: HashSet<&str> = priority.iter().map(String::as_str).collect();
        let addresses: Vec<_> = active.into_iter().filter(|a| !excluded.contains(a.as_str())).collect();

        self.observe_addresses("active", &addresses).await
    }

    pub async fn observe_priority(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let addresses = self.addresses.addresses(self.chain, PerpetualAddressTier::Priority).await?;

        self.observe_addresses("priority", &addresses).await
    }

    async fn observe_addresses(&self, tier: &str, addresses: &[String]) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut total_transactions = 0;
        for address in addresses {
            match self.observe_address(address).await {
                Ok(count) => total_transactions += count,
                Err(error) => error_with_fields!("perpetual_observer", &*error, chain = self.chain.as_ref(), address = address),
            }
        }

        if !addresses.is_empty() {
            info_with_fields!("perpetual_observer", tier = tier, chain = self.chain.as_ref(), addresses = addresses.len(), transactions = total_transactions);
        }

        Ok(addresses.len())
    }

    async fn observe_address(&self, address: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let now = chrono::Utc::now().timestamp() as u64;
        let from_timestamp = self.addresses.checkpoint(self.chain, address).await?.unwrap_or(now);
        let limit = self.config.get_param_usize(&ConfigParamKey::TransactionsRequestLimit(self.chain)).await?;

        let request = TransactionsRequest::new(address.to_string(), limit).with_from_timestamp(Some(from_timestamp));
        let transactions = self.providers.get_transactions_by_address(self.chain, request).await?;

        let payload = TransactionsPayload::new_with_notify(self.chain, vec![], transactions);
        let count = self.stream_producer.publish_transactions(payload).await?;

        self.addresses.set_checkpoint(self.chain, address, now).await?;

        Ok(count)
    }
}
