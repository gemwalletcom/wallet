pub mod model;
pub mod rules;
pub mod store;
#[cfg(test)]
pub(crate) mod testkit;
pub mod tracker;
pub mod tracking_port;

use crate::services::error::GemServiceError;
use crate::services::failures::record;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use primitives::{Asset, AssetId, JobConfiguration, Transaction, TransactionId, TransactionState, TransactionUpdate, Wallet, WalletId};

pub use model::{GemPendingTransaction, GemPostProcessingFailure, GemPostProcessingStep, GemTransactionStateResult, GemTransactionStateUpdate};
pub use store::GemTransactionStateStore;
use tracker::{GemTransactionUpdater, Tracking, poll};
pub use tracking_port::GemTransactionTracking;

use crate::gateway::GemGateway;
use crate::services::assets::GemAssetsService;
use crate::services::balance::GemBalanceService;
use crate::services::nft::GemNftService;
use crate::services::stake::GemStakeService;

#[derive(uniffi::Object)]
pub struct GemTransactionStateService {
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemTransactionStateStore>,
    assets: Arc<GemAssetsService>,
    balance: Arc<GemBalanceService>,
    stake: Arc<GemStakeService>,
    nft: Arc<GemNftService>,
    tracking: Tracking,
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
    ) -> Self {
        Self {
            gateway,
            store,
            assets,
            balance,
            stake,
            nft,
            tracking: Tracking::default(),
        }
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
        self.add_transactions(wallet.id, vec![transaction]).await?;
        Ok(Some(asset))
    }
}

impl GemTransactionStateService {
    pub async fn add_transactions(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        self.store.add_transactions(wallet_id, transactions).await
    }

    pub async fn enable_transaction_assets(&self, wallet_id: WalletId, transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
        let asset_ids = rules::assets_to_enable(&transactions);
        if asset_ids.is_empty() {
            return Ok(());
        }
        self.balance.set_assets_enabled(wallet_id, asset_ids, true).await
    }

    pub async fn update(&self, wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        let update = self.gateway.get_transaction_update(transaction.clone()).await.map_err(|error| error.to_string());
        let previous_state = transaction.state;
        let result = apply(self.store.as_ref(), wallet_id.clone(), transaction.clone(), update, Utc::now()).await?;
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
        record(
            &mut failures,
            GemPostProcessingStep::Balances,
            self.balance.update(wallet_id.clone(), processing.balance_asset_ids),
        )
        .await;
        for chain in processing.stake_chains {
            record(
                &mut failures,
                GemPostProcessingStep::Stake,
                self.stake.sync_wallet(wallet_id.clone(), chain, transaction.from.clone()),
            )
            .await;
        }
        for asset_id in processing.earn_asset_ids {
            record(
                &mut failures,
                GemPostProcessingStep::Earn,
                self.stake.sync_earn_wallet(wallet_id.clone(), asset_id, transaction.from.clone()),
            )
            .await;
        }
        if processing.sync_nfts {
            record(&mut failures, GemPostProcessingStep::Nfts, async { self.nft.sync_wallet(wallet_id).await.map(|_| ()) }).await;
        }
        failures
    }
}

async fn apply(
    store: &dyn GemTransactionStateStore,
    wallet_id: WalletId,
    transaction: Transaction,
    update: Result<TransactionUpdate, String>,
    now: DateTime<Utc>,
) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
    let timed_out = rules::has_timed_out(&transaction, now);
    let update = match update {
        Ok(update) => update,
        Err(_) if timed_out => TransactionUpdate::new_state(TransactionState::Failed),
        Err(msg) => return Err(GemServiceError::Gateway { msg }),
    };
    let (transaction_id, current_state) = match rules::new_hash(&update.changes) {
        Some(hash) => rename(store, &wallet_id, &transaction, hash).await?,
        None => (transaction.id.clone(), transaction.state),
    };
    let next_state = match current_state.merged_with(update.state) {
        state if timed_out && !state.is_completed() => TransactionState::Failed,
        state => state,
    };
    let fields = rules::state_update(next_state, &update.changes).map_err(|error| GemServiceError::Core { msg: error.to_string() })?;
    if next_state == current_state && !fields.has_field_changes() {
        let state = store.get_state(wallet_id, transaction_id.clone()).await?;
        return Ok(state.map(|state| GemTransactionStateResult {
            transaction_id,
            state,
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

async fn rename(store: &dyn GemTransactionStateStore, wallet_id: &WalletId, transaction: &Transaction, hash: String) -> Result<(TransactionId, TransactionState), GemServiceError> {
    let new_transaction_id = TransactionId::new(transaction.asset_id.chain, hash);
    match store.get_state(wallet_id.clone(), new_transaction_id.clone()).await? {
        Some(existing_state) => {
            store.delete_transaction(wallet_id.clone(), transaction.id.clone()).await?;
            Ok((new_transaction_id, existing_state))
        }
        None => {
            store.rename_transaction(wallet_id.clone(), transaction.id.clone(), new_transaction_id.clone()).await?;
            Ok((new_transaction_id, transaction.state))
        }
    }
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
    use num_bigint::BigUint;
    use primitives::{AssetId, Chain, TransactionChange, TransactionMetadata, TransactionSwapMetadata, TransactionType};

    fn swap_metadata(to_value: BigUint) -> TransactionSwapMetadata {
        TransactionSwapMetadata {
            from_asset: AssetId::from_chain(Chain::Ethereum),
            from_value: BigUint::parse_bytes(b"1000000000000000000", 10).unwrap(),
            to_asset: AssetId::from_chain(Chain::Bitcoin),
            to_value,
            provider: Some("thorchain".into()),
        }
    }

    fn transaction(hash: &str, state: TransactionState, created_at: DateTime<Utc>) -> Transaction {
        Transaction::new(
            hash.into(),
            AssetId::from_chain(Chain::Ethereum),
            "from".into(),
            "to".into(),
            None,
            TransactionType::Swap,
            state,
            BigUint::from(1u64),
            AssetId::from_chain(Chain::Ethereum),
            BigUint::from(1000000000000000000u64),
            None,
            serde_json::to_value(swap_metadata(BigUint::parse_bytes(b"10000000000000000000", 10).unwrap())).ok(),
            created_at,
        )
    }

    fn update(state: TransactionState, changes: Vec<TransactionChange>) -> Result<TransactionUpdate, String> {
        Ok(TransactionUpdate::new(state, changes))
    }

    fn apply_update(
        store: &MemoryTransactionStateStore,
        transaction: Transaction,
        update: Result<TransactionUpdate, String>,
        now: DateTime<Utc>,
    ) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        futures::executor::block_on(apply(store, WalletId::Multicoin("wallet".into()), transaction, update, now))
    }

    fn id(hash: &str) -> TransactionId {
        TransactionId::new(Chain::Ethereum, hash.into())
    }

    #[test]
    fn test_in_transit_saves_metadata_and_keeps_polling() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(id("hash"), TransactionState::Pending)]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now),
            update(
                TransactionState::InTransit,
                vec![TransactionChange::Metadata(TransactionMetadata::Swap(swap_metadata(
                    BigUint::parse_bytes(b"9900000000000000000", 10).unwrap(),
                )))],
            ),
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
        let store = MemoryTransactionStateStore::with(vec![(id("hash"), TransactionState::Pending)]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now),
            update(
                TransactionState::InTransit,
                vec![TransactionChange::HashChange {
                    old: "hash".into(),
                    new: "new-hash".into(),
                }],
            ),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result.transaction_id, id("new-hash"));
        assert_eq!(store.renamed.lock().unwrap().as_slice(), &[(id("hash"), id("new-hash"))]);
        assert_eq!(store.updates.lock().unwrap()[0].0, id("new-hash"));
    }

    #[test]
    fn test_hash_change_merges_into_existing_row_without_downgrade() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(id("hash"), TransactionState::Pending), (id("new-hash"), TransactionState::Confirmed)]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now),
            update(
                TransactionState::InTransit,
                vec![TransactionChange::HashChange {
                    old: "hash".into(),
                    new: "new-hash".into(),
                }],
            ),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(
            result,
            GemTransactionStateResult {
                transaction_id: id("new-hash"),
                state: TransactionState::Confirmed,
                failures: Vec::new(),
            }
        );
        assert_eq!(store.deleted.lock().unwrap().as_slice(), &[id("hash")]);
        assert!(store.renamed.lock().unwrap().is_empty());
        assert!(store.updates.lock().unwrap().is_empty());
    }

    #[test]
    fn test_in_transit_is_not_downgraded_to_pending() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(id("hash"), TransactionState::InTransit)]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::InTransit, now),
            update(TransactionState::Pending, vec![]),
            now,
        )
        .unwrap()
        .unwrap();

        assert_eq!(result.state, TransactionState::InTransit);
        assert!(store.updates.lock().unwrap().is_empty());
    }

    #[test]
    fn test_removed_row_stops_polling() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now),
            update(TransactionState::Confirmed, vec![]),
            now,
        )
        .unwrap();

        assert!(result.is_none());
    }

    #[test]
    fn test_status_failure_fails_only_after_timeout() {
        let now = Utc::now();
        let store = MemoryTransactionStateStore::with(vec![(id("hash"), TransactionState::Pending)]);

        let fresh = apply_update(&store, transaction("hash", TransactionState::Pending, now), Err("offline".into()), now);
        assert!(matches!(fresh, Err(GemServiceError::Gateway { .. })));

        let stale = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now - chrono::Duration::hours(2)),
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
        let swap = transaction("hash", TransactionState::Pending, now);

        assert_eq!(rules::post_processing(&swap, TransactionState::Pending, TransactionState::Pending), None);
        assert_eq!(rules::post_processing(&swap, TransactionState::InTransit, TransactionState::InTransit), None);

        let in_transit = rules::post_processing(&swap, TransactionState::Pending, TransactionState::InTransit).unwrap();
        assert_eq!(in_transit.balance_asset_ids.len(), 2);
        assert!(in_transit.stake_chains.is_empty() && in_transit.earn_asset_ids.is_empty() && !in_transit.sync_nfts);

        let mut stake = transaction("hash", TransactionState::Pending, now);
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
        let in_transit = transaction("hash", TransactionState::InTransit, created_at);

        assert_eq!(rules::destination_chain(&in_transit), Some(Chain::Bitcoin));
        assert!(!rules::has_timed_out(&in_transit, now));
        assert!(rules::has_timed_out(&transaction("hash", TransactionState::Pending, created_at), now));
        assert!(!rules::has_timed_out(
            &transaction("hash", TransactionState::Confirmed, now - chrono::Duration::days(30)),
            now
        ));
    }

    #[test]
    fn test_assets_to_enable_skips_hypercore_and_duplicates() {
        let swap = transaction("swap", TransactionState::Pending, Utc::now());
        let hypercore = Transaction::new(
            "perpetual".into(),
            AssetId::from_chain(Chain::HyperCore),
            "from".into(),
            "to".into(),
            None,
            TransactionType::Transfer,
            TransactionState::Pending,
            BigUint::from(1u64),
            AssetId::from_chain(Chain::HyperCore),
            BigUint::from(1u64),
            None,
            None,
            Utc::now(),
        );
        let asset_ids = rules::assets_to_enable(&[swap.clone(), swap, hypercore]);
        assert_eq!(asset_ids.len(), 2);
        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Ethereum)));
        assert!(asset_ids.contains(&AssetId::from_chain(Chain::Bitcoin)));
    }
}
