use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use chrono::{DateTime, Utc};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::swap::{SwapResult, SwapStatus};
use primitives::{Chain, JobConfiguration, Transaction, TransactionId, TransactionState, TransactionSwapMetadata, TransactionType};
use storage::{Database, TransactionFilter, TransactionUpdate, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};
use swapper::cross_chain::{self, DepositAddressMap};
use swapper::swapper::GemSwapper;

use crate::transactions::SwapVaultAddressClient;

#[derive(Clone, Copy)]
pub struct InTransitConfig {
    pub timeout: Duration,
    pub query_limit: i64,
    pub check_interval: JobConfiguration,
}

impl InTransitConfig {
    fn query_limit(&self) -> usize {
        self.query_limit.max(0) as usize
    }

    fn scan_limit(&self) -> i64 {
        self.query_limit.max(0).saturating_mul(i64::from(self.check_interval.max_interval_steps()))
    }
}

struct CheckSchedule {
    next_check_at: DateTime<Utc>,
    interval_ms: u32,
}

pub struct InTransitUpdater {
    database: Database,
    config: InTransitConfig,
    swapper: Arc<GemSwapper>,
    stream_producer: StreamProducer,
    vault_client: SwapVaultAddressClient,
    check_schedules: Mutex<HashMap<TransactionId, CheckSchedule>>,
}

impl InTransitUpdater {
    pub fn new(database: Database, config: InTransitConfig, swapper: Arc<GemSwapper>, stream_producer: StreamProducer, vault_client: SwapVaultAddressClient) -> Self {
        Self {
            database,
            config,
            swapper,
            stream_producer,
            vault_client,
            check_schedules: Mutex::new(HashMap::new()),
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let scan_limit = self.config.scan_limit();
        let transactions = self
            .database
            .run(move |client| client.get_transactions_by_filter(vec![TransactionFilter::States(vec![TransactionState::InTransit])], scan_limit))
            .await?;
        let now = Utc::now();
        let transactions_to_check = {
            let schedules = self.check_schedules();
            transactions
                .iter()
                .filter(|transaction| match schedules.get(&transaction.id) {
                    Some(schedule) => schedule.next_check_at <= now,
                    None => true,
                })
                .take(self.config.query_limit())
                .collect::<Vec<_>>()
        };

        if transactions_to_check.is_empty() {
            return Ok(0);
        }

        let vault_addresses = self.vault_client.get_deposit_address_map().await?;
        let cutoff = now - self.config.timeout;
        let mut updated = 0;

        for transaction in transactions_to_check {
            if self.process_transaction(transaction, now, cutoff, &vault_addresses).await? {
                updated += 1;
            }
        }

        Ok(updated)
    }

    async fn process_transaction(&self, transaction: &Transaction, now: DateTime<Utc>, cutoff: DateTime<Utc>, vault_addresses: &DepositAddressMap) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chain = transaction.id.chain;
        let hash = transaction.id.hash.as_str();
        let elapsed = match (now - transaction.created_at).to_std() {
            Ok(duration) => DurationMs(duration),
            Err(_) => DurationMs(Duration::default()),
        };

        let provider = cross_chain::swap_provider_with_vault_addresses(transaction, vault_addresses);
        let provider_name = provider.as_ref().map(|provider| provider.as_ref().to_string()).unwrap_or_default();
        let result = match provider {
            Some(provider) => match self.swapper.get_swap_result(chain, provider, hash).await {
                Ok(r) => r,
                Err(err) => {
                    error_with_fields!("in_transit check failed", &err as &dyn Error, chain = chain.as_ref(), hash = hash, provider = provider_name, elapsed = elapsed);
                    if transaction.created_at < cutoff {
                        info_with_fields!("in_transit timed out", chain = chain.as_ref(), hash = hash, provider = provider_name, elapsed = elapsed);
                        self.check_schedules().remove(&transaction.id);
                        self.save_and_publish(chain, transaction, TransactionState::Failed, None).await?;
                        return Ok(true);
                    }
                    self.schedule_next_check(transaction, now);
                    return Ok(false);
                }
            },
            None => SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: None,
            },
        };
        let Some((state, metadata)) = resolve_status(&result, transaction.created_at, cutoff) else {
            info_with_fields!("in_transit pending", chain = chain.as_ref(), hash = hash, provider = provider_name, elapsed = elapsed);
            self.schedule_next_check(transaction, now);
            return Ok(false);
        };

        info_with_fields!("in_transit confirmed", chain = chain.as_ref(), hash = hash, state = state.as_ref(), elapsed = elapsed);

        self.check_schedules().remove(&transaction.id);
        let metadata = metadata.and_then(|m| serde_json::to_value(m).ok());
        self.save_and_publish(chain, transaction, state, metadata).await?;
        Ok(true)
    }

    fn schedule_next_check(&self, transaction: &Transaction, now: DateTime<Utc>) {
        let mut schedules = self.check_schedules();
        let current_interval_ms = match schedules.get(&transaction.id) {
            Some(schedule) => schedule.interval_ms,
            None => self.config.check_interval.initial_interval_ms,
        };
        schedules.insert(
            transaction.id.clone(),
            CheckSchedule {
                next_check_at: now + chrono::Duration::milliseconds(i64::from(current_interval_ms)),
                interval_ms: self.config.check_interval.next_interval_ms(current_interval_ms),
            },
        );
    }

    fn check_schedules(&self) -> MutexGuard<'_, HashMap<TransactionId, CheckSchedule>> {
        match self.check_schedules.lock() {
            Ok(schedules) => schedules,
            Err(error) => error.into_inner(),
        }
    }

    async fn save_and_publish(&self, chain: Chain, transaction: &Transaction, state: TransactionState, metadata: Option<serde_json::Value>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let updates = match metadata {
            Some(ref json) => vec![TransactionUpdate::State(state), TransactionUpdate::Kind(TransactionType::Swap), TransactionUpdate::Metadata(json.clone())],
            None => vec![TransactionUpdate::State(state), TransactionUpdate::Kind(TransactionType::Swap)],
        };
        let hash = transaction.id.hash.clone();
        self.database.run(move |client| client.update_transaction(chain.as_ref(), &hash, updates)).await?;

        let transaction = transaction.clone().with_swap_state(state, metadata.clone());
        self.stream_producer.publish_transactions(TransactionsPayload::new_state_change_with_notify(chain, vec![transaction])).await?;
        Ok(())
    }
}

fn resolve_status(result: &SwapResult, created_at: DateTime<Utc>, cutoff: DateTime<Utc>) -> Option<(TransactionState, Option<TransactionSwapMetadata>)> {
    let metadata = result.metadata.clone();
    match result.status.transaction_state() {
        Some(state) => Some((state, metadata)),
        None if created_at < cutoff => Some((TransactionState::Failed, metadata)),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{HOUR, MINUTE};

    #[test]
    fn test_scan_limit_covers_check_interval_window() {
        let config = InTransitConfig {
            timeout: MINUTE,
            query_limit: 100,
            check_interval: JobConfiguration {
                initial_interval_ms: 60_000,
                max_interval_ms: 300_000,
                step_factor: 2.0,
            },
        };

        assert_eq!(config.query_limit(), 100);
        assert_eq!(config.scan_limit(), 500);
    }

    #[test]
    fn test_resolve_status_completed() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Completed,
            ..SwapResult::pending()
        };
        let Some((state, _)) = resolve_status(&result, now, now) else {
            panic!("completed status should resolve");
        };
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_resolve_status_failed() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Failed,
            ..SwapResult::pending()
        };
        let Some((state, _)) = resolve_status(&result, now, now) else {
            panic!("failed status should resolve");
        };
        assert_eq!(state, TransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_refunded() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Refunded,
            ..SwapResult::pending()
        };
        let Some((state, _)) = resolve_status(&result, now, now) else {
            panic!("refunded status should resolve");
        };
        assert_eq!(state, TransactionState::Refunded);
    }

    #[test]
    fn test_resolve_status_pending_within_timeout() {
        let now = Utc::now();
        let cutoff = Utc::now() - HOUR;
        assert!(resolve_status(&SwapResult::pending(), now, cutoff).is_none());
    }

    #[test]
    fn test_resolve_status_pending_past_timeout() {
        let cutoff = Utc::now();
        let created_at = Utc::now() - HOUR * 2;
        let Some((state, _)) = resolve_status(&SwapResult::pending(), created_at, cutoff) else {
            panic!("timed out pending status should resolve");
        };
        assert_eq!(state, TransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_metadata_from_result() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Completed,
            metadata: Some(TransactionSwapMetadata {
                from_asset: "bitcoin".into(),
                from_value: BigUint::from(50_000u64),
                to_asset: "ethereum".into(),
                to_value: BigUint::from(2_500u64),
                provider: Some("thorchain".to_string()),
            }),
            ..SwapResult::pending()
        };
        let Some((_, Some(resolved))) = resolve_status(&result, now, now) else {
            panic!("completed status should include metadata");
        };
        assert_eq!(resolved.from_value, BigUint::from(50_000u64));
    }
}
