use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cacher::{CacheKey, CacherClient};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::{Chain, ConfigParamKey, TransactionId, chain_transaction_timeout};
use settings_chain::{ChainProviders, TransactionIdRequest};
use storage::{ConfigCacher, Database, DatabaseError, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

pub struct PendingTransactionsUpdaterConfig {
    max_age_by_chain: HashMap<Chain, Duration>,
}

impl PendingTransactionsUpdaterConfig {
    pub fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            max_age_by_chain: config.get_param_durations(Chain::all(), ConfigParamKey::TransactionsPendingMaxAge)?,
        })
    }

    fn max_age(&self, chain: Chain) -> Duration {
        self.max_age_by_chain[&chain]
    }

}

pub struct PendingTransactionsUpdater {
    providers: Arc<ChainProviders>,
    cacher: CacherClient,
    stream_producer: StreamProducer,
    database: Database,
    config: PendingTransactionsUpdaterConfig,
}

impl PendingTransactionsUpdater {
    pub fn new(
        providers: Arc<ChainProviders>,
        cacher: CacherClient,
        stream_producer: StreamProducer,
        database: Database,
        config: PendingTransactionsUpdaterConfig,
    ) -> Self {
        Self {
            providers,
            cacher,
            stream_producer,
            database,
            config,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut updated = 0;
        for chain in Chain::all() {
            if !self.has_pending_transactions(chain).await? {
                continue;
            }
            updated += self.update_chain(chain).await?;
        }

        Ok(updated)
    }

    async fn update_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let pending_key = CacheKey::PendingTransactions(chain.as_ref());
        let identifiers = self.cacher.sorted_set_range_with_scores(&pending_key.key(), 0, -1).await?;

        if identifiers.is_empty() {
            return Ok(0);
        }

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        let mut count = 0;

        for (identifier, expires_at) in identifiers {
            if self.process_identifier(chain, &identifier, expires_at, now).await? {
                count += self.remove_pending_transaction(chain, &identifier).await?;
            }
        }

        Ok(count)
    }

    async fn process_identifier(&self, chain: Chain, identifier: &str, expires_at: f64, now: f64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let elapsed_duration = pending_transaction_elapsed(chain, expires_at, now);
        let elapsed = DurationMs(elapsed_duration);
        let transaction_id = TransactionId::new(chain, identifier.to_string());

        if pending_transaction_expired(chain, expires_at, now, self.config.max_age(chain)) {
            info_with_fields!("pending transaction expired", chain = chain.as_ref(), identifier = identifier, elapsed = elapsed);
            return Ok(true);
        }

        if self.database.transactions()?.get_transaction_exists(&transaction_id)? {
            info_with_fields!("pending transaction already stored", chain = chain.as_ref(), identifier = identifier, elapsed = elapsed);
            return Ok(true);
        }

        let start = Instant::now();
        match self.providers.get_transaction_by_hash(TransactionIdRequest::new(chain, identifier.to_string(), None)).await {
            Ok(Some(transaction)) => {
                info_with_fields!(
                    "pending transaction load success",
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = elapsed,
                    latency = DurationMs(start.elapsed())
                );
                self.stream_producer
                    .publish_transactions(TransactionsPayload::new_with_notify(chain, vec![], vec![transaction]))
                    .await?;
                Ok(true)
            }
            Ok(None) => {
                info_with_fields!(
                    "pending transaction not loaded",
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = elapsed,
                    latency = DurationMs(start.elapsed())
                );
                Ok(false)
            }
            Err(err) => {
                error_with_fields!(
                    "pending transaction load failed",
                    &*err,
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = elapsed,
                    latency = DurationMs(start.elapsed()),
                    dropped = true
                );
                Ok(true)
            }
        }
    }

    async fn remove_pending_transaction(&self, chain: Chain, identifier: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.cacher
            .remove_from_sorted_set_cached(CacheKey::PendingTransactions(chain.as_ref()), &[identifier.to_string()])
            .await
    }

    async fn has_pending_transactions(&self, chain: Chain) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let pending_key = CacheKey::PendingTransactions(chain.as_ref());
        let pending_count = self.cacher.sorted_set_card(&pending_key.key()).await?;
        Ok(pending_count > 0)
    }
}

fn pending_transaction_elapsed(chain: Chain, expires_at: f64, now: f64) -> Duration {
    let timeout = f64::from(chain_transaction_timeout(chain)) / 1000.0;
    let added_at = expires_at - timeout;
    Duration::from_secs_f64((now - added_at).max(0.0))
}

fn pending_transaction_expired(chain: Chain, expires_at: f64, now: f64, max_age: Duration) -> bool {
    expires_at <= now || pending_transaction_elapsed(chain, expires_at, now) >= max_age
}

#[cfg(test)]
mod tests {
    use super::{pending_transaction_elapsed, pending_transaction_expired};
    use std::time::Duration;

    use primitives::{Chain, chain_transaction_timeout};

    #[test]
    fn test_pending_transaction_elapsed_uses_added_at() {
        let chain = Chain::Ethereum;
        let expires_at = 10_000.0;
        let now = expires_at - f64::from(chain_transaction_timeout(chain) / 1000) + 42.0;

        assert_eq!(pending_transaction_elapsed(chain, expires_at, now), Duration::from_secs(42));
    }

    #[test]
    fn test_pending_transaction_elapsed_is_zero_before_added_at() {
        let chain = Chain::Xrp;
        let expires_at = 10_000.0;
        let now = expires_at - f64::from(chain_transaction_timeout(chain) / 1000) - 1.0;

        assert_eq!(pending_transaction_elapsed(chain, expires_at, now), Duration::ZERO);
    }

    #[test]
    fn test_pending_transaction_expires_at_max_age() {
        let chain = Chain::Bitcoin;
        let max_age = Duration::from_secs(3 * 24 * 60 * 60);
        let timeout = f64::from(chain_transaction_timeout(chain)) / 1000.0;
        let added_at = 10_000.0;
        let expires_at = added_at + timeout;

        assert!(!pending_transaction_expired(chain, expires_at, added_at + max_age.as_secs_f64() - 1.0, max_age));
        assert!(pending_transaction_expired(chain, expires_at, added_at + max_age.as_secs_f64(), max_age));
    }
}
