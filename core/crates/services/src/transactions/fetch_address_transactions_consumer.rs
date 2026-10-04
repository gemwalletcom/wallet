use std::error::Error;
use std::sync::Arc;

use crate::ConfigCacher;
use async_trait::async_trait;
use cacher::{ThrottleCacher, ThrottledTask};
use chain_providers::{ChainProviders, TransactionsRequest, TransactionsResult};
use config_keys::ConfigParamKey;
use streamer::{ChainAddressPayload, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

pub struct FetchAddressTransactionsConsumer {
    pub providers: ChainProviders,
    pub producer: Arc<dyn StreamProducerQueue>,
    pub throttle: Arc<dyn ThrottleCacher>,
    pub config: Arc<ConfigCacher>,
}

impl FetchAddressTransactionsConsumer {
    pub fn new(providers: ChainProviders, producer: Arc<dyn StreamProducerQueue>, throttle: Arc<dyn ThrottleCacher>, config: Arc<ConfigCacher>) -> Self {
        Self { providers, producer, throttle, config }
    }
}

#[async_trait]
impl MessageConsumer<ChainAddressPayload, usize> for FetchAddressTransactionsConsumer {
    async fn should_consume(&self, payload: &ChainAddressPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.throttle
            .try_start(ThrottledTask::FetchAddressTransactions {
                chain: payload.value.chain.as_ref(),
                address: &payload.value.address,
            })
            .await
    }
    async fn consume(&self, payload: ChainAddressPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let chain = payload.value.chain;
        let limit = self.config.get_param_usize(&ConfigParamKey::TransactionsRequestLimit(chain)).await?;
        let transactions_result = self.providers.get_transactions_by_address_result(chain, TransactionsRequest::new(payload.value.address, limit)).await?;
        match transactions_result {
            TransactionsResult::Transactions(transactions) => self.producer.publish_transactions(TransactionsPayload::new(chain, transactions)).await,
            TransactionsResult::TransactionRequests(transactions) => self.producer.publish_fetch_transactions(transactions).await,
        }
    }
}
