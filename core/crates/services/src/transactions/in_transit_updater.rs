use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use config_keys::{ConfigKey, ConfigParamKey};
use futures::{StreamExt, TryStreamExt, stream};
use gem_tracing::{DurationMs, error_with_fields, info_with_fields};
use primitives::swap::{SwapResult, SwapResultRequest, SwapStatus};
use primitives::{Chain, JobConfiguration, Transaction, TransactionState, TransactionSwapMetadata};
use storage::{DatabaseError, TransactionFilter};

use super::repository::Repository;
use streamer::{StreamProducerQueue, TransactionsPayload};
use swapper::SwapperProvider;
use swapper::cross_chain::{self, DepositAddressMap};
use swapper::swapper::GemSwapper;

use crate::config::repository::Repository as ConfigurationRepository;
use crate::transactions::{CheckSchedule, SwapVaultAddressClient, TransactionQueue, TransactionQueueGroup, TransactionQueueMetrics, swap_result_metadata, swap_state_updates, transaction_with_swap_state};

struct InTransitConfig {
    timeout: Duration,
    query_limit: i64,
    check_interval: JobConfiguration,
    enabled_providers: BTreeSet<SwapperProvider>,
}

impl InTransitConfig {
    async fn from_config(config: &dyn ConfigurationRepository) -> Result<Self, DatabaseError> {
        let mut enabled_providers = BTreeSet::new();
        for provider in SwapperProvider::all() {
            if config.get_param_bool(&ConfigParamKey::TransactionInTransitProviderEnable(provider)).await? {
                enabled_providers.insert(provider);
            }
        }
        Ok(Self {
            timeout: config.get_duration(ConfigKey::TransactionInTransitTimeout).await?,
            query_limit: config.get_i64(ConfigKey::TransactionInTransitQueryLimit).await?,
            check_interval: JobConfiguration {
                initial_interval_ms: config.get_duration(ConfigKey::TransactionTimerInTransitUpdate).await?.as_millis() as u32,
                max_interval_ms: config.get_duration(ConfigKey::TransactionInTransitMaxCheckInterval).await?.as_millis() as u32,
                step_factor: config.get_f64(ConfigKey::TransactionInTransitCheckIntervalFactor).await? as f32,
            },
            enabled_providers,
        })
    }

    fn query_limit(&self) -> usize {
        self.query_limit.max(0) as usize
    }

    fn scan_limit(&self) -> i64 {
        self.query_limit.max(0).saturating_mul(i64::from(self.check_interval.max_interval_steps()))
    }
}

pub struct InTransitUpdater {
    repository: Arc<dyn Repository>,
    config: Arc<dyn ConfigurationRepository>,
    swapper: Arc<GemSwapper>,
    stream_producer: Arc<dyn StreamProducerQueue>,
    vault_client: SwapVaultAddressClient,
    metrics: Arc<dyn TransactionQueueMetrics>,
    schedule: CheckSchedule,
}

impl InTransitUpdater {
    pub(crate) fn new(
        repository: Arc<dyn Repository>,
        config: Arc<dyn ConfigurationRepository>,
        swapper: Arc<GemSwapper>,
        stream_producer: Arc<dyn StreamProducerQueue>,
        vault_client: SwapVaultAddressClient,
        metrics: Arc<dyn TransactionQueueMetrics>,
        schedule: CheckSchedule,
    ) -> Self {
        Self {
            repository,
            config,
            swapper,
            stream_producer,
            vault_client,
            metrics,
            schedule,
        }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let config = InTransitConfig::from_config(self.config.as_ref()).await?;
        let vault_addresses = self.vault_client.get_deposit_address_map().await?;
        let now = Utc::now();
        let (transactions_to_check, counts) = due_in_transit_transactions(self.repository.as_ref(), &self.schedule, &config, now, &vault_addresses).await?;
        self.metrics.record_queue(TransactionQueue::InTransit, counts);
        let cutoff = now - config.timeout;

        stream::iter(&transactions_to_check)
            .then(|transaction| self.update_transaction(transaction, now, cutoff, &vault_addresses, &config))
            .try_fold(0, |total, updated| async move { Ok(total + usize::from(updated)) })
            .await
    }

    async fn update_transaction(&self, transaction: &Transaction, now: DateTime<Utc>, cutoff: DateTime<Utc>, vault_addresses: &DepositAddressMap, config: &InTransitConfig) -> Result<bool, Box<dyn Error + Send + Sync>> {
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
                    self.schedule.schedule_next(&transaction.id, &config.check_interval, elapsed_duration, now).await?;
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
            self.schedule.schedule_next(&transaction.id, &config.check_interval, elapsed_duration, now).await?;
            return Ok(false);
        };

        info_with_fields!("in_transit completed", chain = chain.as_ref(), hash = hash, provider = provider_name, state = state.as_ref(), elapsed = elapsed);
        if transaction.created_at >= cutoff {
            let to_chain = metadata.as_ref().map(|metadata| metadata.to_asset.chain).or_else(|| transaction.swap_metadata().map(|metadata| metadata.to_asset.chain));
            self.metrics.record_completion(TransactionQueue::InTransit, TransactionQueueGroup::new(chain, provider), to_chain, elapsed_duration);
        }

        self.schedule.remove(&transaction.id).await?;
        let metadata = swap_result_metadata(transaction, metadata, state.swap_status().charges_referral_fee());
        self.save_and_publish(chain, transaction, state, metadata).await?;
        Ok(true)
    }

    async fn save_and_publish(&self, chain: Chain, transaction: &Transaction, state: TransactionState, metadata: Option<serde_json::Value>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let transaction = transaction_with_swap_state(transaction.clone(), state, metadata.clone());
        let updates = swap_state_updates(&transaction, metadata.as_ref());
        self.repository
            .update_transactions(vec![TransactionFilter::Chain(chain.as_ref().to_string()), TransactionFilter::Hash(transaction.id.hash.clone())], updates)
            .await?;

        self.stream_producer.publish_transactions(TransactionsPayload::new_state_change_with_notify(chain, vec![transaction])).await?;
        Ok(())
    }
}

async fn due_in_transit_transactions(
    repository: &dyn Repository,
    schedule: &CheckSchedule,
    config: &InTransitConfig,
    now: DateTime<Utc>,
    vault_addresses: &DepositAddressMap,
) -> Result<(Vec<Transaction>, BTreeMap<TransactionQueueGroup, usize>), Box<dyn Error + Send + Sync>> {
    let mut transactions_to_check = Vec::new();
    let mut counts = BTreeMap::new();
    let mut after = None;
    while transactions_to_check.len() < config.query_limit() {
        let mut filters = vec![TransactionFilter::States(vec![TransactionState::InTransit])];
        if let Some(cursor) = after {
            filters.push(cursor);
        }
        let transactions = repository.get_transactions(filters, config.scan_limit()).await?;
        let Some(last) = transactions.last() else {
            break;
        };
        after = Some(TransactionFilter::After(last.created_at.naive_utc(), last.id.clone()));
        let is_last_page = transactions.len() < config.scan_limit() as usize;
        for (group, count) in in_transit_counts(&transactions, vault_addresses) {
            *counts.entry(group).or_default() += count;
        }
        let enabled = transactions
            .into_iter()
            .filter(|transaction| cross_chain::in_transit_swap_provider(transaction, vault_addresses).is_none_or(|provider| config.enabled_providers.contains(&provider)))
            .collect::<Vec<_>>();
        let ids = enabled.iter().map(|transaction| transaction.id.clone()).collect::<Vec<_>>();
        let due = schedule.due(&ids, now).await?;
        let remaining = config.query_limit() - transactions_to_check.len();
        transactions_to_check.extend(enabled.into_iter().filter(|transaction| due.contains(&transaction.id)).take(remaining));
        if is_last_page {
            break;
        }
    }
    Ok((transactions_to_check, counts))
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
    use crate::testkit::{MemoryCheckScheduleCacher, MemoryConfigRepository, MemoryTransactionsRepository};
    use cacher::CheckScheduleCacher;
    use num_bigint::BigUint;
    use primitives::{HOUR, MINUTE, SwapProvider, TransactionId};

    #[tokio::test]
    async fn test_provider_checks_are_enabled_by_default() {
        let config = MemoryConfigRepository::new();
        let in_transit = InTransitConfig::from_config(&config).await.unwrap();

        assert_eq!(in_transit.enabled_providers, SwapperProvider::all().into_iter().collect());
    }

    #[tokio::test]
    async fn test_disabled_relay_does_not_block_other_providers_past_scan_limit() {
        let config = MemoryConfigRepository::new()
            .with_value(&ConfigParamKey::TransactionInTransitProviderEnable(SwapProvider::Relay).key(), "false")
            .with_value(ConfigKey::TransactionInTransitQueryLimit.as_ref(), "2")
            .with_value(ConfigKey::TransactionTimerInTransitUpdate.as_ref(), "1m")
            .with_value(ConfigKey::TransactionInTransitMaxCheckInterval.as_ref(), "1m");
        let config = InTransitConfig::from_config(&config).await.unwrap();
        assert_eq!(config.enabled_providers, SwapperProvider::all().into_iter().filter(|provider| *provider != SwapProvider::Relay).collect());

        let created_at = DateTime::<Utc>::UNIX_EPOCH;
        let mut transactions = (0..6)
            .map(|index| Transaction {
                id: TransactionId::new(Chain::Ethereum, format!("{index:04}")),
                to: "relay_vault".to_string(),
                state: TransactionState::InTransit,
                created_at,
                ..Transaction::mock()
            })
            .collect::<Vec<_>>();
        transactions.push(Transaction {
            id: TransactionId::new(Chain::Ethereum, "0006".to_string()),
            to: "other_address".to_string(),
            metadata: Some(
                serde_json::to_value(TransactionSwapMetadata::new(
                    Chain::Ethereum.as_asset_id(),
                    BigUint::from(1u32),
                    Chain::Solana.as_asset_id(),
                    BigUint::from(1u32),
                    SwapProvider::Relay,
                ))
                .unwrap(),
            ),
            ..transactions[0].clone()
        });
        for (hash, to) in [("0007", "mayan_vault"), ("0008", "bridgers_vault"), ("0009", "unknown")] {
            transactions.push(Transaction {
                id: TransactionId::new(Chain::Ethereum, hash.to_string()),
                to: to.to_string(),
                ..transactions[0].clone()
            });
        }
        let repository = MemoryTransactionsRepository::new(transactions, vec![]);
        let cacher = Arc::new(MemoryCheckScheduleCacher::default());
        let now = Utc::now();
        cacher
            .set_next_check("in_transit", TransactionId::new(Chain::Ethereum, "0007".to_string()).to_string(), (now + HOUR).timestamp() as f64)
            .await
            .unwrap();
        let schedule = CheckSchedule::new(cacher.clone(), TransactionQueue::InTransit);
        let vault_addresses = DepositAddressMap::from([
            ("relay_vault".to_string(), SwapProvider::Relay),
            ("mayan_vault".to_string(), SwapProvider::Mayan),
            ("bridgers_vault".to_string(), SwapProvider::Bridgers),
        ]);

        let (selected, _) = due_in_transit_transactions(&repository, &schedule, &config, now, &vault_addresses).await.unwrap();

        assert_eq!(selected.iter().map(|transaction| transaction.id.hash.as_str()).collect::<Vec<_>>(), vec!["0008", "0009"]);
        assert_eq!(cacher.checked_ids(), vec!["ethereum_0007", "ethereum_0008", "ethereum_0009"]);
    }

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
            enabled_providers: SwapperProvider::all().into_iter().collect(),
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
