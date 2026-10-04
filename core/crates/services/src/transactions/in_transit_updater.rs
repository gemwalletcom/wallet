use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt, stream};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::swap::{SwapResult, SwapResultRequest, SwapStatus};
use primitives::{Chain, JobConfiguration, Transaction, TransactionState, TransactionSwapMetadata};
use storage::{Database, TransactionFilter, TransactionsRepository};
use streamer::{StreamProducerQueue, TransactionsPayload};
use swapper::cross_chain::{self, DepositAddressMap};
use swapper::swapper::GemSwapper;

use crate::transactions::{CheckSchedule, SwapVaultAddressClient, TransactionQueue, TransactionQueueGroup, TransactionQueueMetrics, swap_result_metadata, swap_state_updates};

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

pub struct InTransitUpdater {
    database: Database,
    config: InTransitConfig,
    swapper: Arc<GemSwapper>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    vault_client: SwapVaultAddressClient,
    metrics: Arc<dyn TransactionQueueMetrics>,
    schedule: CheckSchedule,
}

impl InTransitUpdater {
    pub fn new(
        database: Database,
        config: InTransitConfig,
        swapper: Arc<GemSwapper>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        vault_client: SwapVaultAddressClient,
        metrics: Arc<dyn TransactionQueueMetrics>,
        schedule: CheckSchedule,
    ) -> Self {
        Self {
            database,
            config,
            swapper,
            stream_producer,
            vault_client,
            metrics,
            schedule,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let scan_limit = self.config.scan_limit();
        let transactions = self
            .database
            .run(move |client| client.get_transactions_by_filter(vec![TransactionFilter::States(vec![TransactionState::InTransit])], scan_limit))
            .await?;
        let vault_addresses = self.vault_client.get_deposit_address_map().await?;
        self.metrics.record_queue(TransactionQueue::InTransit, in_transit_counts(&transactions, &vault_addresses));

        let now = Utc::now();
        let ids = transactions.iter().map(|transaction| transaction.id.clone()).collect::<Vec<_>>();
        let due = self.schedule.due(&ids, now).await?;
        let transactions_to_check = transactions.iter().filter(|transaction| due.contains(&transaction.id)).take(self.config.query_limit()).collect::<Vec<_>>();
        let cutoff = now - self.config.timeout;

        stream::iter(transactions_to_check)
            .then(|transaction| self.update_transaction(transaction, now, cutoff, &vault_addresses))
            .try_fold(0, |total, updated| async move { Ok(total + usize::from(updated)) })
            .await
    }

    async fn update_transaction(&self, transaction: &Transaction, now: DateTime<Utc>, cutoff: DateTime<Utc>, vault_addresses: &DepositAddressMap) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let chain = transaction.id.chain;
        let hash = transaction.id.hash.as_str();
        let elapsed_duration = (now - transaction.created_at).to_std().unwrap_or_default();
        let elapsed = DurationMs(elapsed_duration);

        let provider = cross_chain::in_transit_swap_provider(transaction, vault_addresses);
        let provider_name = provider.as_ref().map(|provider| provider.as_ref().to_string()).unwrap_or_default();
        let result = match provider {
            Some(provider) => match self.swapper.get_swap_result(provider, &SwapResultRequest::from(transaction)).await {
                Ok(r) => r,
                Err(error) => {
                    error_with_fields!("in_transit error", &error as &dyn Error, chain = chain.as_ref(), hash = hash, provider = provider_name);
                    if transaction.created_at < cutoff {
                        info_with_fields!("in_transit expired", chain = chain.as_ref(), hash = hash, provider = provider_name, elapsed = elapsed);
                        self.schedule.remove(&transaction.id).await?;
                        self.save_and_publish(chain, transaction, TransactionState::Failed, None).await?;
                        return Ok(true);
                    }
                    self.schedule.schedule_next(&transaction.id, &self.config.check_interval, elapsed_duration, now).await?;
                    return Ok(false);
                }
            },
            None => SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: None,
            },
        };
        let Some((state, metadata)) = final_swap_state(&result, transaction.created_at, cutoff) else {
            info_with_fields!("in_transit waiting", chain = chain.as_ref(), hash = hash, provider = provider_name);
            self.schedule.schedule_next(&transaction.id, &self.config.check_interval, elapsed_duration, now).await?;
            return Ok(false);
        };

        info_with_fields!("in_transit completed", chain = chain.as_ref(), hash = hash, provider = provider_name, state = state.as_ref(), elapsed = elapsed);
        if transaction.created_at >= cutoff {
            let to_chain = metadata.as_ref().map(|metadata| metadata.to_asset.chain).or_else(|| transaction.swap_metadata().map(|metadata| metadata.to_asset.chain));
            self.metrics.record_completion(TransactionQueue::InTransit, TransactionQueueGroup::new(chain, provider), to_chain, elapsed_duration);
        }

        self.schedule.remove(&transaction.id).await?;
        let metadata = swap_result_metadata(transaction, metadata);
        self.save_and_publish(chain, transaction, state, metadata).await?;
        Ok(true)
    }

    async fn save_and_publish(&self, chain: Chain, transaction: &Transaction, state: TransactionState, metadata: Option<serde_json::Value>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let updates = swap_state_updates(state, metadata.as_ref());
        let hash = transaction.id.hash.clone();
        self.database.run(move |client| client.update_transaction(chain.as_ref(), &hash, updates)).await?;

        let transaction = transaction.clone().with_swap_state(state, metadata.clone());
        self.stream_producer.publish_transactions(TransactionsPayload::new_state_change_with_notify(chain, vec![transaction])).await?;
        Ok(())
    }
}

fn final_swap_state(result: &SwapResult, created_at: DateTime<Utc>, cutoff: DateTime<Utc>) -> Option<(TransactionState, Option<TransactionSwapMetadata>)> {
    let metadata = result.metadata.clone();
    match result.status.transaction_state() {
        Some(state) => Some((state, metadata)),
        None if created_at < cutoff => Some((TransactionState::Failed, metadata)),
        None => None,
    }
}

fn in_transit_counts(transactions: &[Transaction], vault_addresses: &DepositAddressMap) -> BTreeMap<TransactionQueueGroup, usize> {
    transactions
        .iter()
        .map(|transaction| TransactionQueueGroup::new(transaction.id.chain, cross_chain::in_transit_swap_provider(transaction, vault_addresses)))
        .fold(BTreeMap::new(), |mut counts, group| {
            *counts.entry(group).or_default() += 1;
            counts
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_bigint::BigUint;
    use primitives::{HOUR, MINUTE, SwapProvider};

    #[test]
    fn test_in_transit_counts() {
        let vault_deposit = Transaction {
            to: "vault".to_string(),
            ..Transaction::mock()
        };
        let solana = Transaction::mock_with_params(primitives::AssetId::from_chain(Chain::Solana), primitives::TransactionType::Swap, BigUint::from(1u32));
        let vault_addresses = DepositAddressMap::from([("vault".to_string(), SwapProvider::Thorchain)]);

        assert_eq!(
            in_transit_counts(&[vault_deposit.clone(), vault_deposit, solana], &vault_addresses),
            BTreeMap::from([(TransactionQueueGroup::new(Chain::Ethereum, Some(SwapProvider::Thorchain)), 2), (TransactionQueueGroup::new(Chain::Solana, None), 1),])
        );
        assert_eq!(in_transit_counts(&[], &vault_addresses), BTreeMap::new());
    }

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
    fn test_final_swap_state_completed() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Completed,
            ..SwapResult::pending()
        };
        let Some((state, _)) = final_swap_state(&result, now, now) else {
            panic!("completed status should resolve");
        };
        assert_eq!(state, TransactionState::Confirmed);
    }

    #[test]
    fn test_final_swap_state_failed() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Failed,
            ..SwapResult::pending()
        };
        let Some((state, _)) = final_swap_state(&result, now, now) else {
            panic!("failed status should resolve");
        };
        assert_eq!(state, TransactionState::Failed);
    }

    #[test]
    fn test_final_swap_state_refunded() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Refunded,
            ..SwapResult::pending()
        };
        let Some((state, _)) = final_swap_state(&result, now, now) else {
            panic!("refunded status should resolve");
        };
        assert_eq!(state, TransactionState::Refunded);
    }

    #[test]
    fn test_final_swap_state_pending_within_timeout() {
        let now = Utc::now();
        let cutoff = Utc::now() - HOUR;
        assert!(final_swap_state(&SwapResult::pending(), now, cutoff).is_none());
    }

    #[test]
    fn test_final_swap_state_pending_past_timeout() {
        let cutoff = Utc::now();
        let created_at = Utc::now() - HOUR * 2;
        let Some((state, _)) = final_swap_state(&SwapResult::pending(), created_at, cutoff) else {
            panic!("timed out pending status should resolve");
        };
        assert_eq!(state, TransactionState::Failed);
    }

    #[test]
    fn test_final_swap_state_metadata_from_result() {
        let now = Utc::now();
        let result = SwapResult {
            status: SwapStatus::Completed,
            metadata: Some(TransactionSwapMetadata::new("bitcoin".into(), BigUint::from(50_000u64), "ethereum".into(), BigUint::from(2_500u64), SwapProvider::Thorchain)),
            ..SwapResult::pending()
        };
        let Some((_, Some(resolved))) = final_swap_state(&result, now, now) else {
            panic!("completed status should include metadata");
        };
        assert_eq!(resolved.from_value, BigUint::from(50_000u64));
    }
}
