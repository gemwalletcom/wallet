use std::error::Error;
use std::sync::Arc;

use chain_providers::ChainProviders;
use streamer::{FetchBlocksPayload, StreamProducerQueue, TransactionsPayload, consumer::MessageConsumer};

pub struct FetchBlocksConsumer {
    pub providers: ChainProviders,
    pub stream_producer: Arc<dyn StreamProducerQueue>,
}

impl FetchBlocksConsumer {
    pub fn new(providers: ChainProviders, stream_producer: Arc<dyn StreamProducerQueue>) -> Self {
        Self { providers, stream_producer }
    }
}

impl MessageConsumer<FetchBlocksPayload, usize> for FetchBlocksConsumer {
    async fn should_consume(&self, _payload: &FetchBlocksPayload) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }
    async fn consume(&self, payload: FetchBlocksPayload) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let blocks = vec![payload.block];
        let transactions = self.providers.get_transactions_in_blocks(payload.chain, blocks.clone()).await?;
        let payload = TransactionsPayload::new_with_notify(payload.chain, blocks, transactions);
        self.stream_producer.publish_transactions(payload).await
    }
}
