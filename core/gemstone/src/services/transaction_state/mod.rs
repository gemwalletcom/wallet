pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;
pub mod tracker;

use crate::services::error::GemServiceError;
use crate::services::failures::record;
use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use primitives::{Asset, AssetId, JobConfiguration, Transaction, TransactionId, TransactionState, TransactionUpdate, Wallet, WalletId};

pub use model::{GemPendingTransaction, GemPostProcessingFailure, GemPostProcessingStep, GemTransactionStateResult, GemTransactionStateUpdate};
pub use store::GemTransactionStateStore;
use tracker::{GemTransactionUpdater, Tracking, poll};

use crate::gateway::GemGateway;
use crate::payment::GemPaymentService;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::nft::GemNftService;
use crate::services::stake::GemStakeService;

#[uniffi::export(with_foreign)]
pub trait GemTransactionStatusService: Send + Sync {
    fn track(&self, wallet_id: WalletId, transactions: Vec<Transaction>);
}

#[derive(uniffi::Object)]
pub struct GemTransactionStateService {
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemTransactionStateStore>,
    assets: Arc<GemAssetsService>,
    balance: Arc<GemBalanceService>,
    stake: Arc<GemStakeService>,
    nft: Arc<GemNftService>,
    payments: Arc<GemPaymentService>,
    tracking: Tracking,
    status: OnceLock<Arc<dyn GemTransactionStatusService>>,
}

#[uniffi::export]
impl GemTransactionStateService {
    #[uniffi::constructor]
    pub fn new(
        gateway: Arc<GemGateway>,
        store: Arc<dyn GemTransactionStateStore>,
        assets: Arc<GemAssetsService>,
        balance: Arc<GemBalanceService>,
        stake: Arc<GemStakeService>,
        nft: Arc<GemNftService>,
        payments: Arc<GemPaymentService>,
    ) -> Self {
        Self {
            gateway,
            store,
            assets,
            balance,
            stake,
            nft,
            payments,
            tracking: Tracking::default(),
            status: OnceLock::new(),
        }
    }

    pub fn set_status(&self, status: Arc<dyn GemTransactionStatusService>) {
        let _ = self.status.set(status);
    }

    pub async fn track_pending(&self) -> Result<(), GemServiceError> {
        let pending = self.store.get_pending_transactions().await?;
        let tracked = pending.into_iter().map(|pending| self.track_transaction(pending.wallet.id, pending.transaction));
        futures::future::join_all(tracked).await;
        Ok(())
    }

    pub async fn track(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.enable_transaction_assets(wallet_id.clone(), transactions.clone()).await?;
        let tracked = transactions.into_iter().map(|transaction| self.track_transaction(wallet_id.clone(), transaction));
        futures::future::join_all(tracked).await;
        Ok(())
    }

    pub fn stop_tracking(&self) {
        self.tracking.cancel();
    }

    pub async fn add_notification_transaction(&self, wallet: Wallet, asset_id: AssetId, transaction: Transaction) -> Result<Option<Asset>, GemServiceError> {
        let Some(asset) = self.assets.open_wallet_asset(wallet.clone(), asset_id).await? else {
            return Ok(None);
        };
        self.add_transactions(wallet.id.clone(), vec![transaction.clone()]).await?;
        if let Some(status) = self.status.get() {
            status.track(wallet.id, vec![transaction]);
        }
        Ok(Some(asset))
    }
}

impl GemTransactionStateService {
    pub async fn add_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.store.add_transactions(wallet_id, transactions).await
    }

    async fn enable_transaction_assets(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        let asset_ids = rules::assets_to_enable(&transactions);
        if asset_ids.is_empty() {
            return Ok(());
        }
        self.balance.set_assets_enabled(wallet_id, asset_ids, true).await
    }

    pub async fn update(&self, wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        let update = match rules::payment_link(&transaction) {
            Some(link) => self.payments.transaction_update(transaction.hash(), &link).await.map_err(|error| error.to_string()),
            None => self.gateway.get_transaction_update(transaction.clone()).await.map_err(|error| error.to_string()),
        };
        let previous_state = transaction.state;
        let result = merge_update(self.store.as_ref(), wallet_id.clone(), transaction.clone(), update, Utc::now()).await?;
        let Some(mut result) = result else {
            return Ok(None);
        };
        result.failures = self.post_process(wallet_id, &transaction, previous_state, result.state).await;
        Ok(Some(result))
    }

    async fn track_transaction(&self, wallet_id: WalletId, transaction: Transaction) {
        let configuration = JobConfiguration::transaction_state(transaction.asset_id.chain);
        poll(self, self.store.as_ref(), &self.tracking, configuration, wallet_id, transaction).await;
    }

    async fn post_process(&self, wallet_id: WalletId, transaction: &Transaction, previous_state: TransactionState, state: TransactionState) -> Vec<GemPostProcessingFailure> {
        let Some(processing) = rules::post_processing(transaction, previous_state, state) else {
            return Vec::new();
        };
        let mut failures = Vec::new();
        record(&mut failures, GemPostProcessingStep::Balances, self.balance.update(wallet_id.clone(), processing.balance_asset_ids)).await;
        for chain in processing.stake_chains {
            record(&mut failures, GemPostProcessingStep::Stake, self.stake.sync_wallet(wallet_id.clone(), chain, transaction.from.clone())).await;
        }
        for asset_id in processing.earn_asset_ids {
            record(&mut failures, GemPostProcessingStep::Earn, self.stake.sync_earn_wallet(wallet_id.clone(), asset_id, transaction.from.clone())).await;
        }
        if processing.sync_nfts {
            record(&mut failures, GemPostProcessingStep::Nfts, async { self.nft.sync_wallet(wallet_id).await.map(|_| ()) }).await;
        }
        failures
    }
}

async fn merge_update(store: &dyn GemTransactionStateStore, wallet_id: WalletId, transaction: Transaction, update: Result<TransactionUpdate, String>, now: DateTime<Utc>) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
    let timed_out = rules::has_timed_out(&transaction, now);
    let update = match update {
        Ok(update) => update,
        Err(_) if timed_out => TransactionUpdate::new_state(TransactionState::Failed),
        Err(msg) => return Err(GemServiceError::Gateway { msg }),
    };
    let transaction_id = match rules::new_hash(&update.changes) {
        Some(hash) => {
            let new_transaction_id = TransactionId::new(transaction.id.chain, hash.clone());
            store.update_transaction_hash(wallet_id.clone(), transaction.id.clone(), hash).await?;
            new_transaction_id
        }
        None => transaction.id.clone(),
    };
    let Some(current_state) = store.get_state(wallet_id.clone(), transaction_id.clone()).await? else {
        return Ok(None);
    };
    let next_state = match current_state.merged_with(update.state) {
        state if timed_out && !state.is_completed() => TransactionState::Failed,
        state => state,
    };
    let fields = rules::state_update(next_state, &update.changes, &transaction).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
    if next_state == current_state && !fields.has_field_changes() {
        return Ok(Some(GemTransactionStateResult {
            transaction_id,
            state: current_state,
            failures: Vec::new(),
        }));
    }
    if !store.update_transaction(wallet_id, transaction_id.clone(), fields).await? {
        return Ok(None);
    }
    Ok(Some(GemTransactionStateResult {
        transaction_id,
        state: next_state,
        failures: Vec::new(),
    }))
}

#[async_trait]
impl GemTransactionUpdater for GemTransactionStateService {
    async fn update(&self, wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        GemTransactionStateService::update(self, wallet_id, transaction).await
    }
}

#[cfg(test)]
mod tests {
    use super::testkit::MemoryTransactionStateStore;
    use super::*;
    use crate::services::asset_discovery::testkit::DiscoveryTestkit;
    use crate::services::assets::GemAssetStore;
    use crate::services::assets::rules::default_asset_basic;
    use futures::executor::block_on;
    use num_bigint::{BigInt, BigUint};
    use primitives::{AssetId, Chain, TransactionChange, TransactionMetadata, TransactionSwapMetadata, TransactionType};

    #[test]
    fn test_a_transaction_that_arrived_by_push_is_tracked_without_the_app_asking() {
        block_on(async {
            let testkit = DiscoveryTestkit::with_status(200);
            let wallet = Wallet::mock();
            let asset = Asset::from_chain(Chain::Ethereum);
            testkit.asset_store.save_assets(vec![default_asset_basic(asset.clone())]).await.unwrap();
            let transaction = Transaction {
                asset_id: asset.id.clone(),
                ..Transaction::mock()
            };

            let opened = testkit.state.add_notification_transaction(wallet, asset.id.clone(), transaction.clone()).await.unwrap();

            assert_eq!(opened, Some(asset));
            assert_eq!(*testkit.status.tracked.lock().unwrap(), vec![vec![transaction]]);
        });
    }

    fn merge_polled_update(store: &MemoryTransactionStateStore, transaction: Transaction, update: Result<TransactionUpdate, String>, now: DateTime<Utc>) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        futures::executor::block_on(merge_update(store, WalletId::Multicoin("wallet".into()), transaction, update, now))
    }

    #[test]
    fn test_in_transit_saves_metadata_and_keeps_polling() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Pending)]);

        let result = merge_polled_update(
            &store,
            Transaction { created_at: now, ..Transaction::mock_swap() },
            Ok(TransactionUpdate::new(
                TransactionState::InTransit,
                vec![TransactionChange::Metadata(TransactionMetadata::Swap(TransactionSwapMetadata {
                    to_value: BigUint::parse_bytes(b"9900000000000000000", 10).unwrap(),
                    ..TransactionSwapMetadata::mock()
                }))],
            )),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result.state, TransactionState::InTransit);
        let (_, saved) = store.updates.lock().unwrap()[0].clone();
        assert_eq!(saved.state, TransactionState::InTransit);
        assert!(saved.metadata.unwrap().contains("9900000000000000000"));
    }

    #[test]
    fn test_hash_change_renames_when_no_existing_row() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Pending)]);

        let result = merge_polled_update(
            &store,
            Transaction { created_at: now, ..Transaction::mock_swap() },
            Ok(TransactionUpdate::new(TransactionState::InTransit, vec![TransactionChange::HashChange { old: "hash".into(), new: "new-hash".into() }])),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result.transaction_id, TransactionId::mock("new-hash"));
        assert_eq!(store.hash_updates.lock().unwrap().as_slice(), &[(TransactionId::mock("hash"), TransactionId::mock("new-hash"))]);
        assert_eq!(store.updates.lock().unwrap()[0].0, TransactionId::mock("new-hash"));
    }

    #[test]
    fn test_hash_change_merges_into_existing_row_without_downgrade() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Pending), (TransactionId::mock("new-hash"), TransactionState::Confirmed)]);

        let result = merge_polled_update(
            &store,
            Transaction { created_at: now, ..Transaction::mock_swap() },
            Ok(TransactionUpdate::new(TransactionState::InTransit, vec![TransactionChange::HashChange { old: "hash".into(), new: "new-hash".into() }])),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result, GemTransactionStateResult::mock(TransactionId::mock("new-hash"), TransactionState::Confirmed));
        assert_eq!(store.deleted.lock().unwrap().as_slice(), &[]);
        assert_eq!(store.hash_updates.lock().unwrap().as_slice(), &[(TransactionId::mock("hash"), TransactionId::mock("new-hash"))]);
        assert_eq!(store.states.lock().unwrap().as_slice(), &[(TransactionId::mock("new-hash"), TransactionState::Confirmed)]);
        assert!(store.updates.lock().unwrap().is_empty());
    }

    #[test]
    fn test_hash_change_is_idempotent() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Pending)]);
        for hash in ["hash", "new-hash", "new-hash"] {
            let result = merge_polled_update(
                &store,
                Transaction { created_at: now, ..Transaction::mock_swap() },
                Ok(TransactionUpdate::new(TransactionState::Confirmed, vec![TransactionChange::HashChange { old: "hash".into(), new: hash.into() }])),
                now,
            )
            .unwrap()
            .unwrap();

            assert_eq!(result.transaction_id, TransactionId::mock(hash));
            assert_eq!(result.state, TransactionState::Confirmed);
            assert_eq!(store.states.lock().unwrap().as_slice(), &[(TransactionId::mock(hash), TransactionState::Confirmed)]);
            assert_eq!(store.deleted.lock().unwrap().as_slice(), &[]);
        }
    }

    #[test]
    fn test_in_transit_is_not_downgraded_to_pending() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::InTransit)]);

        let result = merge_polled_update(
            &store,
            Transaction {
                state: TransactionState::InTransit,
                created_at: now,
                ..Transaction::mock_swap()
            },
            Ok(TransactionUpdate::new(TransactionState::Pending, vec![])),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result.state, TransactionState::InTransit);
        assert!(store.updates.lock().unwrap().is_empty());
    }

    #[test]
    fn test_a_delayed_poll_merges_against_the_stored_state_not_its_own_snapshot() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Confirmed)]);
        let polled = Transaction {
            state: TransactionState::Pending,
            created_at: now,
            ..Transaction::mock_swap()
        };

        let unchanged = merge_polled_update(&store, polled.clone(), Ok(TransactionUpdate::new(TransactionState::Pending, vec![])), now).unwrap().unwrap();

        assert_eq!(unchanged.state, TransactionState::Confirmed);
        assert!(store.updates.lock().unwrap().is_empty(), "a confirmed row is not written back to pending");

        let with_fee = merge_polled_update(&store, polled, Ok(TransactionUpdate::new(TransactionState::Pending, vec![TransactionChange::NetworkFee(BigInt::from(21000))])), now)
            .unwrap()
            .unwrap();

        assert_eq!(with_fee.state, TransactionState::Confirmed);
        let (_, saved) = store.updates.lock().unwrap()[0].clone();
        assert_eq!(saved.state, TransactionState::Confirmed, "late fields land on the stored state, not the snapshot's");
    }

    #[test]
    fn test_removed_row_stops_polling() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![]);

        for changes in [vec![], vec![TransactionChange::HashChange { old: "hash".into(), new: "new-hash".into() }]] {
            let result = merge_polled_update(&store, Transaction { created_at: now, ..Transaction::mock_swap() }, Ok(TransactionUpdate::new(TransactionState::Confirmed, changes)), now).unwrap();

            assert_eq!(result, None);
        }
    }

    #[test]
    fn test_status_failure_fails_only_after_timeout() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(TransactionId::mock("hash"), TransactionState::Pending)]);

        let fresh = merge_polled_update(&store, Transaction { created_at: now, ..Transaction::mock_swap() }, Err("offline".into()), now);
        assert!(matches!(fresh, Err(GemServiceError::Gateway { .. })));

        let stale = merge_polled_update(
            &store,
            Transaction {
                created_at: now - chrono::Duration::hours(2),
                ..Transaction::mock_swap()
            },
            Err("offline".into()),
            now,
        )
        .unwrap()
        .unwrap();
        assert_eq!(stale.state, TransactionState::Failed);
    }

    #[test]
    fn test_post_processing_by_state_transition() {
        let now = Utc::now();
        let swap = Transaction { created_at: now, ..Transaction::mock_swap() };

        assert_eq!(rules::post_processing(&swap, TransactionState::Pending, TransactionState::Pending), None);
        assert_eq!(rules::post_processing(&swap, TransactionState::InTransit, TransactionState::InTransit), None);

        let in_transit = rules::post_processing(&swap, TransactionState::Pending, TransactionState::InTransit).unwrap();
        assert_eq!(in_transit.balance_asset_ids.len(), 2);
        assert!(in_transit.stake_chains.is_empty() && in_transit.earn_asset_ids.is_empty() && !in_transit.sync_nfts);

        let mut stake = Transaction { created_at: now, ..Transaction::mock_swap() };
        stake.transaction_type = TransactionType::StakeFreeze;
        stake.metadata = None;
        let completed = rules::post_processing(&stake, TransactionState::Pending, TransactionState::Confirmed).unwrap();
        assert_eq!(completed.stake_chains, vec![Chain::Ethereum]);
        assert_eq!(completed.balance_asset_ids, vec![AssetId::from_chain(Chain::Ethereum)]);

        let mut nft = stake.clone();
        nft.transaction_type = TransactionType::TransferNFT;
        assert!(rules::post_processing(&nft, TransactionState::Pending, TransactionState::Failed).unwrap().sync_nfts);

        let mut earn = stake.clone();
        earn.transaction_type = TransactionType::EarnDeposit;
        let completed = rules::post_processing(&earn, TransactionState::Pending, TransactionState::Confirmed).unwrap();
        assert_eq!(completed.earn_asset_ids, vec![AssetId::from_chain(Chain::Ethereum)]);
        assert!(completed.stake_chains.is_empty());
    }

    #[test]
    fn test_in_transit_timeout_uses_destination_chain() {
        let now = Utc::now();
        let created_at = now - chrono::Duration::hours(2);
        let in_transit = Transaction {
            state: TransactionState::InTransit,
            created_at,
            ..Transaction::mock_swap()
        };

        assert_eq!(rules::destination_chain(&in_transit), Some(Chain::Bitcoin));
        assert!(!rules::has_timed_out(&in_transit, now));
        assert!(rules::has_timed_out(&Transaction { created_at, ..Transaction::mock_swap() }, now));
        assert!(!rules::has_timed_out(
            &Transaction {
                state: TransactionState::Confirmed,
                created_at: now - chrono::Duration::days(30),
                ..Transaction::mock_swap()
            },
            now
        ));
    }

    #[test]
    fn test_assets_to_enable_skips_hypercore_and_duplicates() {
        let swap = Transaction {
            id: TransactionId::mock("swap"),
            created_at: Utc::now(),
            ..Transaction::mock_swap()
        };
        let hypercore = Transaction::mock_with_params(AssetId::from_chain(Chain::HyperCore), TransactionType::Transfer, BigUint::from(1u64));
        let asset_ids = rules::assets_to_enable(&[swap.clone(), swap, hypercore]);
        assert_eq!(asset_ids.len(), 2);
        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Bitcoin)));
    }
}
