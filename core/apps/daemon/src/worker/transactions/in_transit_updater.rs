use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use chrono::{DateTime, NaiveDateTime, Utc};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::swap::{SwapResult, SwapStatus};
use primitives::{Chain, JobConfiguration, TransactionSwapMetadata, TransactionType};
use storage::models::TransactionRow;
use storage::{Database, TransactionFilter, TransactionState, TransactionUpdate, TransactionsRepository};
use streamer::{StreamProducer, StreamProducerQueue, TransactionsPayload};
use swapper::cross_chain::{self, DepositAddressMap};
use swapper::swapper::GemSwapper;

use crate::client::SwapVaultAddressClient;

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
    check_schedules: Mutex<HashMap<i64, CheckSchedule>>,
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
        let transactions = self
            .database
            .transactions()?
            .get_transactions_by_filter(vec![TransactionFilter::States(vec![TransactionState::InTransit])], self.config.scan_limit())?;
        let now = Utc::now();
        let transactions_to_check = {
            let schedules = self.check_schedules();
            transactions
                .iter()
                .filter(|row| match schedules.get(&row.id) {
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
        let cutoff = (now - self.config.timeout).naive_utc();
        let mut updated = 0;

        for transaction in transactions_to_check {
            if self.process_transaction(transaction, now, cutoff, &vault_addresses).await? {
                updated += 1;
            }
        }

        Ok(updated)
    }

    async fn process_transaction(
        &self,
        row: &TransactionRow,
        now: DateTime<Utc>,
        cutoff: NaiveDateTime,
        vault_addresses: &DepositAddressMap,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chain = row.chain();
        let transaction = row.as_primitive(row.get_addresses());
        let elapsed = match (now.naive_utc() - row.created_at).to_std() {
            Ok(duration) => DurationMs(duration),
            Err(_) => DurationMs(Duration::default()),
        };

        let provider = cross_chain::swap_provider_with_vault_addresses(&transaction, vault_addresses);
        let provider_name = provider.as_ref().map(|provider| provider.as_ref().to_string()).unwrap_or_default();
        let result = match provider {
            Some(provider) => match self.swapper.get_swap_result(chain, provider, &row.hash).await {
                Ok(r) => r,
                Err(err) => {
                    error_with_fields!(
                        "in_transit check failed",
                        &err as &dyn Error,
                        chain = chain.as_ref(),
                        hash = row.hash,
                        provider = provider_name,
                        elapsed = elapsed
                    );
                    if row.created_at < cutoff {
                        info_with_fields!("in_transit timed out", chain = chain.as_ref(), hash = row.hash, provider = provider_name, elapsed = elapsed);
                        self.check_schedules().remove(&row.id);
                        self.save_and_publish(chain, row, &TransactionState::Failed, None).await?;
                        return Ok(true);
                    }
                    self.schedule_next_check(row, now);
                    return Ok(false);
                }
            },
            None => SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
            },
        };
        let Some((state, metadata)) = resolve_status(&result, row.created_at, cutoff) else {
            info_with_fields!("in_transit pending", chain = chain.as_ref(), hash = row.hash, provider = provider_name, elapsed = elapsed);
            self.schedule_next_check(row, now);
            return Ok(false);
        };

        info_with_fields!("in_transit confirmed", chain = chain.as_ref(), hash = row.hash, state = state.as_ref(), elapsed = elapsed);

        self.check_schedules().remove(&row.id);
        let metadata = metadata.and_then(|m| serde_json::to_value(m).ok());
        self.save_and_publish(chain, row, &state, metadata).await?;
        Ok(true)
    }

    fn schedule_next_check(&self, row: &TransactionRow, now: DateTime<Utc>) {
        let mut schedules = self.check_schedules();
        let current_interval_ms = match schedules.get(&row.id) {
            Some(schedule) => schedule.interval_ms,
            None => self.config.check_interval.initial_interval_ms,
        };
        schedules.insert(
            row.id,
            CheckSchedule {
                next_check_at: now + chrono::Duration::milliseconds(i64::from(current_interval_ms)),
                interval_ms: self.config.check_interval.next_interval_ms(current_interval_ms),
            },
        );
    }

    fn check_schedules(&self) -> std::sync::MutexGuard<'_, HashMap<i64, CheckSchedule>> {
        match self.check_schedules.lock() {
            Ok(schedules) => schedules,
            Err(error) => error.into_inner(),
        }
    }

    async fn save_and_publish(
        &self,
        chain: Chain,
        row: &TransactionRow,
        state: &TransactionState,
        metadata: Option<serde_json::Value>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let updates = match metadata {
            Some(ref json) => vec![
                TransactionUpdate::State(state.clone()),
                TransactionUpdate::Kind(TransactionType::Swap.into()),
                TransactionUpdate::Metadata(json.clone()),
            ],
            None => vec![TransactionUpdate::State(state.clone()), TransactionUpdate::Kind(TransactionType::Swap.into())],
        };
        self.database.transactions()?.update_transaction(chain.as_ref(), &row.hash, updates)?;

        let transaction = row.as_primitive(row.get_addresses()).with_swap_state(state.clone().into(), metadata.clone());
        self.stream_producer
            .publish_transactions(TransactionsPayload::new_state_change_with_notify(chain, vec![transaction]))
            .await?;
        Ok(())
    }
}

fn resolve_status(result: &SwapResult, created_at: NaiveDateTime, cutoff: NaiveDateTime) -> Option<(TransactionState, Option<TransactionSwapMetadata>)> {
    let metadata = result.metadata.clone();
    match result.status.transaction_state() {
        Some(state) => Some((state.into(), metadata)),
        None if created_at < cutoff => Some((TransactionState::Failed, metadata)),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::TransactionState as PrimitiveTransactionState;

    fn swap_result(status: SwapStatus, metadata: Option<TransactionSwapMetadata>) -> SwapResult {
        SwapResult { status, metadata }
    }

    fn swap_metadata(provider: &str, from_value: &str, to_value: &str) -> TransactionSwapMetadata {
        TransactionSwapMetadata {
            from_asset: "bitcoin".into(),
            from_value: from_value.to_string(),
            to_asset: "ethereum".into(),
            to_value: to_value.to_string(),
            provider: Some(provider.to_string()),
        }
    }

    #[test]
    fn test_scan_limit_covers_check_interval_window() {
        let config = InTransitConfig {
            timeout: Duration::from_secs(60),
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
        let now = Utc::now().naive_utc();
        let Some((state, _)) = resolve_status(&swap_result(SwapStatus::Completed, None), now, now) else {
            panic!("completed status should resolve");
        };
        assert_eq!(*state, PrimitiveTransactionState::Confirmed);
    }

    #[test]
    fn test_resolve_status_failed() {
        let now = Utc::now().naive_utc();
        let Some((state, _)) = resolve_status(&swap_result(SwapStatus::Failed, None), now, now) else {
            panic!("failed status should resolve");
        };
        assert_eq!(*state, PrimitiveTransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_pending_within_timeout() {
        let now = Utc::now().naive_utc();
        let cutoff = (Utc::now() - Duration::from_secs(3600)).naive_utc();
        assert!(resolve_status(&swap_result(SwapStatus::Pending, None), now, cutoff).is_none());
    }

    #[test]
    fn test_resolve_status_pending_past_timeout() {
        let cutoff = Utc::now().naive_utc();
        let created_at = (Utc::now() - Duration::from_secs(7200)).naive_utc();
        let Some((state, _)) = resolve_status(&swap_result(SwapStatus::Pending, None), created_at, cutoff) else {
            panic!("timed out pending status should resolve");
        };
        assert_eq!(*state, PrimitiveTransactionState::Failed);
    }

    #[test]
    fn test_resolve_status_metadata_from_result() {
        let now = Utc::now().naive_utc();
        let metadata = swap_metadata("thorchain", "50000", "2500");
        let Some((_, Some(resolved))) = resolve_status(&swap_result(SwapStatus::Completed, Some(metadata)), now, now) else {
            panic!("completed status should include metadata");
        };
        assert_eq!(resolved.from_value, "50000");
    }
}
