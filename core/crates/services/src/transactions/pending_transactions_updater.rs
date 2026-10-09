use std::collections::{BTreeMap, HashMap};
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::ConfigCacher;
use crate::transactions::{CheckSchedule, TransactionQueue, TransactionQueueGroup, TransactionQueueMetrics};
use cacher::PendingTransactionsCacher;
use chain_providers::{ChainProviders, TransactionIdRequest};
use chrono::{DateTime, Utc};
use config_keys::{ConfigKey, ConfigParamKey};
use futures::{StreamExt, TryStreamExt, future::try_join_all, stream};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::{Chain, JobConfiguration, TransactionId, chain_transaction_timeout};
use storage::DatabaseError;

use super::repository::Repository;
use streamer::{StreamProducerQueue, TransactionsPayload};

pub struct PendingTransactionsUpdaterConfig {
    error_max_age_by_chain: HashMap<Chain, Duration>,
    check_interval: JobConfiguration,
    chain_concurrency: usize,
}

impl PendingTransactionsUpdaterConfig {
    pub async fn from_config(config: &ConfigCacher) -> Result<Self, DatabaseError> {
        Ok(Self {
            error_max_age_by_chain: config.get_param_durations(Chain::all(), ConfigParamKey::TransactionsPendingErrorMaxAge).await?,
            check_interval: JobConfiguration {
                initial_interval_ms: config.get_duration(ConfigKey::TransactionPendingTimer).await?.as_millis() as u32,
                max_interval_ms: config.get_duration(ConfigKey::TransactionPendingMaxCheckInterval).await?.as_millis() as u32,
                step_factor: config.get_f64(ConfigKey::TransactionPendingCheckIntervalFactor).await? as f32,
            },
            chain_concurrency: config.get_usize(ConfigKey::TransactionPendingChainConcurrency).await?.max(1),
        })
    }

    fn error_max_age(&self, chain: Chain) -> Duration {
        self.error_max_age_by_chain[&chain]
    }

    fn check_interval(&self, chain: Chain) -> JobConfiguration {
        self.check_interval.with_block_time(chain)
    }
}

pub struct PendingTransactionsUpdater {
    providers: Arc<ChainProviders>,
    pending: Arc<dyn PendingTransactionsCacher>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    repository: Arc<dyn Repository>,
    config: PendingTransactionsUpdaterConfig,
    metrics: Arc<dyn TransactionQueueMetrics>,
    schedule: CheckSchedule,
}

impl PendingTransactionsUpdater {
    pub(crate) fn new(
        providers: Arc<ChainProviders>,
        pending: Arc<dyn PendingTransactionsCacher>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        repository: Arc<dyn Repository>,
        config: PendingTransactionsUpdaterConfig,
        metrics: Arc<dyn TransactionQueueMetrics>,
        schedule: CheckSchedule,
    ) -> Self {
        Self {
            providers,
            pending,
            stream_producer,
            repository,
            config,
            metrics,
            schedule,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let pending_counts = self.pending_counts().await?;
        let chains = pending_counts.keys().copied().collect::<Vec<_>>();
        let groups = pending_counts.into_iter().map(|(chain, count)| (TransactionQueueGroup::new(chain, None), count)).collect();
        self.metrics.record_queue(TransactionQueue::Pending, groups);

        stream::iter(chains)
            .map(|chain| self.update_chain(chain))
            .buffer_unordered(self.config.chain_concurrency)
            .try_fold(0, |total, count| async move { Ok(total + count) })
            .await
    }

    async fn pending_counts(&self) -> Result<BTreeMap<Chain, usize>, Box<dyn Error + Send + Sync>> {
        let counts = try_join_all(Chain::all().into_iter().map(|chain| async move { Ok::<_, Box<dyn Error + Send + Sync>>((chain, self.pending.pending_count(chain).await?)) })).await?;
        Ok(counts.into_iter().filter(|(_, count)| *count > 0).collect())
    }

    async fn update_chain(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let pending = self
            .pending
            .pending(chain)
            .await?
            .into_iter()
            .map(|(identifier, expires_at)| (TransactionId::new(chain, identifier), expires_at))
            .collect::<Vec<_>>();
        let ids = pending.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>();
        let now = Utc::now();
        let due = self.schedule.due(&ids, now).await?;

        stream::iter(pending.into_iter().filter(|(id, _)| due.contains(id)))
            .then(|(id, expires_at)| self.check_pending_transaction(id, expires_at, now))
            .try_fold(0, |total, removed| async move { Ok(total + removed) })
            .await
    }

    async fn check_pending_transaction(&self, id: TransactionId, expires_at: f64, now: DateTime<Utc>) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let now_seconds = now.timestamp_millis() as f64 / 1000.0;
        if self.update_pending_transaction(id.chain, &id.hash, expires_at, now_seconds).await? {
            self.schedule.remove(&id).await?;
            return self.pending.remove_pending(id.chain, &id.hash).await;
        }
        let elapsed = pending_transaction_elapsed(id.chain, expires_at, now_seconds);
        self.schedule.schedule_next(&id, &self.config.check_interval(id.chain), elapsed, now).await?;
        Ok(0)
    }

    async fn update_pending_transaction(&self, chain: Chain, identifier: &str, expires_at: f64, now: f64) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let elapsed_duration = pending_transaction_elapsed(chain, expires_at, now);
        let elapsed = DurationMs(elapsed_duration);
        let transaction_id = TransactionId::new(chain, identifier.to_string());

        if pending_transaction_expired(expires_at, now) {
            info_with_fields!("pending expired", chain = chain.as_ref(), identifier = identifier, elapsed = elapsed);
            return Ok(true);
        }

        if self.repository.get_transaction_exists(transaction_id).await? {
            info_with_fields!("pending exists", chain = chain.as_ref(), identifier = identifier);
            return Ok(true);
        }

        let start = Instant::now();
        match self.providers.get_transaction_by_hash(TransactionIdRequest::new(chain, identifier.to_string(), None)).await {
            Ok(Some(transaction)) => {
                info_with_fields!("pending loaded", chain = chain.as_ref(), identifier = identifier, elapsed = elapsed, latency = DurationMs(start.elapsed()));
                self.metrics.record_completion(TransactionQueue::Pending, TransactionQueueGroup::new(chain, None), None, elapsed_duration);
                self.stream_producer.publish_transactions(TransactionsPayload::new_with_notify(chain, vec![], vec![transaction])).await?;
                Ok(true)
            }
            Ok(None) => {
                info_with_fields!("pending waiting", chain = chain.as_ref(), identifier = identifier, latency = DurationMs(start.elapsed()));
                Ok(false)
            }
            Err(error) => {
                error_with_fields!("pending error", &*error, chain = chain.as_ref(), identifier = identifier, latency = DurationMs(start.elapsed()));
                Ok(pending_transaction_error_expired(elapsed_duration, self.config.error_max_age(chain)))
            }
        }
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

#[cfg(test)]
mod tests {
    use super::{pending_transaction_elapsed, pending_transaction_error_expired, pending_transaction_expired};
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
}
