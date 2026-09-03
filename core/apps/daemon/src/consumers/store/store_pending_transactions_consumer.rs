use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use gem_tracing::info_with_fields;
use primitives::{TransactionId, chain_transaction_timeout};
use streamer::consumer::MessageConsumer;

pub struct StorePendingTransactionsConsumer {
    cacher: CacherClient,
}

impl StorePendingTransactionsConsumer {
    pub fn new(cacher: CacherClient) -> Self {
        Self { cacher }
    }
}

#[async_trait]
impl MessageConsumer<TransactionId, usize> for StorePendingTransactionsConsumer {
    async fn should_process(&self, _payload: &TransactionId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(true)
    }

    async fn process(&self, payload: TransactionId) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_at = now.saturating_add(u64::from(chain_transaction_timeout(payload.chain)) / 1000) as f64;
        let identifier = payload.hash;
        let key = CacheKey::PendingTransactions(payload.chain.as_ref());
        self.cacher.add_to_sorted_set_cached(key, &[(identifier.clone(), expires_at)]).await?;
        self.cacher
            .add_to_sorted_set_cached(CacheKey::PendingTransactionAttempts(payload.chain.as_ref()), &[(identifier, now as f64)])
            .await?;
        info_with_fields!("stored pending transaction", transaction_id = transaction_id.as_str());
        Ok(1)
    }
}
