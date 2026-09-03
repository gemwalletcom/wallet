use std::collections::{BTreeSet, HashMap};
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cacher::{CacheKey, CacherClient};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::{Chain, ConfigParamKey, TransactionId, chain_transaction_timeout};
use serde::{Deserialize, Serialize};
use settings_chain::{ChainProviders, TransactionIdRequest};
use storage::{ConfigCacher, Database, DatabaseError, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};

const RETRY_INITIAL_DELAY: Duration = Duration::from_secs(30);
const RETRY_MAX_DELAY: Duration = Duration::from_secs(60 * 60);

#[derive(Deserialize, Serialize)]
struct PendingTransactionRetry {
    failure_count: u32,
    next_attempt_at: f64,
}

pub struct PendingTransactionsUpdaterConfig {
    error_max_age_by_chain: HashMap<Chain, Duration>,
}

impl PendingTransactionsUpdaterConfig {
    pub fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            error_max_age_by_chain: config.get_param_durations(Chain::all(), ConfigParamKey::TransactionsPendingErrorMaxAge)?,
        })
    }

    fn error_max_age(&self, chain: Chain) -> Duration {
        self.error_max_age_by_chain[&chain]
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
    pub fn new(providers: Arc<ChainProviders>, cacher: CacherClient, stream_producer: StreamProducer, database: Database, config: PendingTransactionsUpdaterConfig) -> Self {
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
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f64();
        self.initialize_retry_schedule(chain, now).await?;

        let pending_key = CacheKey::PendingTransactions(chain.as_ref()).key();
        let attempts_key = CacheKey::PendingTransactionAttempts(chain.as_ref()).key();
        let mut identifiers = BTreeSet::new();
        identifiers.extend(self.cacher.sorted_set_range_by_score_all(&pending_key, f64::NEG_INFINITY, now).await?);
        identifiers.extend(self.cacher.sorted_set_range_by_score_all(&attempts_key, f64::NEG_INFINITY, now).await?);
        let mut count = 0;

        for identifier in identifiers {
            let Some(expires_at) = self.cacher.sorted_set_score(&pending_key, &identifier).await? else {
                self.remove_retry_schedule(chain, &identifier).await?;
                continue;
            };
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

        if pending_transaction_expired(expires_at, now) {
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
                self.schedule_retry(chain, identifier, expires_at, now, 0).await?;
                Ok(false)
            }
            Err(err) => {
                error_with_fields!(
                    "pending transaction load failed",
                    &*err,
                    chain = chain.as_ref(),
                    identifier = identifier,
                    elapsed = elapsed,
                    latency = DurationMs(start.elapsed())
                );
                if pending_transaction_error_expired(elapsed_duration, self.config.error_max_age(chain)) {
                    return Ok(true);
                }
                let retry_key = CacheKey::PendingTransactionRetry(chain.as_ref(), identifier, retry_ttl(expires_at, now));
                let failure_count = self
                    .cacher
                    .get_cached_optional::<PendingTransactionRetry>(retry_key)
                    .await?
                    .map(|retry| retry.failure_count.saturating_add(1))
                    .unwrap_or(1);
                self.schedule_retry(chain, identifier, expires_at, now, failure_count).await?;
                Ok(false)
            }
        }
    }

    async fn initialize_retry_schedule(&self, chain: Chain, now: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
        let initialized_key = CacheKey::PendingTransactionAttemptsInitialized(chain.as_ref());
        if self.cacher.get_cached_optional::<bool>(initialized_key).await?.unwrap_or(false) {
            return Ok(());
        }

        let pending_key = CacheKey::PendingTransactions(chain.as_ref()).key();
        let attempts = self
            .cacher
            .sorted_set_range_with_scores(&pending_key, 0, -1)
            .await?
            .into_iter()
            .map(|(identifier, _)| (identifier, now))
            .collect::<Vec<_>>();
        self.cacher
            .add_to_sorted_set_cached_if_missing(CacheKey::PendingTransactionAttempts(chain.as_ref()), &attempts)
            .await?;
        self.cacher.set_cached(CacheKey::PendingTransactionAttemptsInitialized(chain.as_ref()), &true).await?;
        Ok(())
    }

    async fn schedule_retry(&self, chain: Chain, identifier: &str, expires_at: f64, now: f64, failure_count: u32) -> Result<(), Box<dyn Error + Send + Sync>> {
        let next_attempt_at = pending_transaction_next_attempt_at(expires_at, now, failure_count);
        let retry = PendingTransactionRetry { failure_count, next_attempt_at };
        self.cacher
            .set_cached(CacheKey::PendingTransactionRetry(chain.as_ref(), identifier, retry_ttl(expires_at, now)), &retry)
            .await?;
        self.cacher
            .add_to_sorted_set_cached(CacheKey::PendingTransactionAttempts(chain.as_ref()), &[(identifier.to_string(), next_attempt_at)])
            .await?;
        Ok(())
    }

    async fn remove_pending_transaction(&self, chain: Chain, identifier: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let count = self
            .cacher
            .remove_from_sorted_set_cached(CacheKey::PendingTransactions(chain.as_ref()), &[identifier.to_string()])
            .await?;
        self.remove_retry_schedule(chain, identifier).await?;
        Ok(count)
    }

    async fn remove_retry_schedule(&self, chain: Chain, identifier: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.cacher
            .remove_from_sorted_set_cached(CacheKey::PendingTransactionAttempts(chain.as_ref()), &[identifier.to_string()])
            .await?;
        self.cacher.delete(&CacheKey::PendingTransactionRetry(chain.as_ref(), identifier, 1).key()).await?;
        Ok(())
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

fn pending_transaction_expired(expires_at: f64, now: f64) -> bool {
    expires_at <= now
}

fn pending_transaction_error_expired(elapsed: Duration, error_max_age: Duration) -> bool {
    elapsed >= error_max_age
}

fn pending_transaction_retry_delay(failure_count: u32) -> Duration {
    let exponent = failure_count.saturating_sub(1).min(31);
    RETRY_INITIAL_DELAY.saturating_mul(2_u32.pow(exponent)).min(RETRY_MAX_DELAY)
}

fn pending_transaction_next_attempt_at(expires_at: f64, now: f64, failure_count: u32) -> f64 {
    (now + pending_transaction_retry_delay(failure_count).as_secs_f64()).min(expires_at)
}

fn retry_ttl(expires_at: f64, now: f64) -> u64 {
    (expires_at - now).ceil().max(1.0) as u64
}

#[cfg(test)]
mod tests {
    use super::{
        RETRY_INITIAL_DELAY, RETRY_MAX_DELAY, pending_transaction_elapsed, pending_transaction_error_expired, pending_transaction_expired, pending_transaction_next_attempt_at,
        pending_transaction_retry_delay, retry_ttl,
    };
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
    fn test_pending_transaction_uses_stored_expiry() {
        let expires_at = 10_000.0;

        assert!(!pending_transaction_expired(expires_at, expires_at - 1.0));
        assert!(pending_transaction_expired(expires_at, expires_at));
    }

    #[test]
    fn test_pending_transaction_error_uses_configured_max_age() {
        let max_age = Duration::from_secs(3 * 24 * 60 * 60);

        assert!(!pending_transaction_error_expired(max_age - Duration::from_secs(1), max_age));
        assert!(pending_transaction_error_expired(max_age, max_age));
    }

    #[test]
    fn test_pending_transaction_retry_backoff() {
        assert_eq!(pending_transaction_retry_delay(0), RETRY_INITIAL_DELAY);
        assert_eq!(pending_transaction_retry_delay(1), RETRY_INITIAL_DELAY);
        assert_eq!(pending_transaction_retry_delay(2), Duration::from_secs(60));
        assert_eq!(pending_transaction_retry_delay(3), Duration::from_secs(120));
        assert_eq!(pending_transaction_retry_delay(20), RETRY_MAX_DELAY);
        assert_eq!(pending_transaction_retry_delay(u32::MAX), RETRY_MAX_DELAY);
    }

    #[test]
    fn test_retry_ttl_preserves_expiry() {
        assert_eq!(pending_transaction_next_attempt_at(1_100.0, 1_000.0, 1), 1_030.0);
        assert_eq!(pending_transaction_next_attempt_at(1_020.0, 1_000.0, 1), 1_020.0);
        assert_eq!(retry_ttl(1_060.0, 1_000.0), 60);
        assert_eq!(retry_ttl(999.0, 1_000.0), 1);
    }
}
