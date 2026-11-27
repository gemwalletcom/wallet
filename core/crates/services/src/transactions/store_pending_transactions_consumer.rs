use cacher::CacherClient;
use primitives::unix_seconds;
use std::error::Error;

use gem_tracing::info_with_fields;
use primitives::{TransactionId, chain_transaction_timeout};
use streamer::consumer::MessageConsumer;

pub struct StorePendingTransactionsConsumer {
    pending: CacherClient,
}

impl StorePendingTransactionsConsumer {
    pub fn new(pending: CacherClient) -> Self {
        Self { pending }
    }
}

impl MessageConsumer<TransactionId, usize> for StorePendingTransactionsConsumer {
    async fn should_consume(&self, _payload: &TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn consume(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        let expires_at = unix_seconds()?.saturating_add(chain_transaction_timeout(payload.chain).as_secs()) as f64;
        self.pending.add_pending(payload.chain, payload.hash, expires_at).await?;
        info_with_fields!("pending added", transaction_id = transaction_id.as_str());
        Ok(1)
    }
}
