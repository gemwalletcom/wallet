pub mod model;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use primitives::{Transaction, TransactionId, TransactionState, TransactionUpdate, WalletId};

pub use model::{GemTransactionStateResult, GemTransactionStateUpdate};
pub use store::GemTransactionStateStore;

use crate::gateway::GemGateway;

#[derive(uniffi::Object)]
pub struct GemTransactionStateService {
    gateway: Arc<GemGateway>,
    store: Arc<dyn GemTransactionStateStore>,
}

#[uniffi::export]
impl GemTransactionStateService {
    #[uniffi::constructor]
    pub fn new(gateway: Arc<GemGateway>, store: Arc<dyn GemTransactionStateStore>) -> Self {
        Self { gateway, store }
    }

    pub async fn update(&self, wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
        let update = self.gateway.get_transaction_update(transaction.clone()).await.map_err(|error| error.to_string());
        apply(self.store.as_ref(), wallet_id, transaction, update, Utc::now()).await
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
        Err(msg) => return Err(GemServiceError::Status { msg }),
    };
    let (transaction_id, current_state) = match rules::new_hash(&update.changes) {
        Some(hash) => rename(store, &wallet_id, &transaction, hash).await?,
        None => (transaction.id.clone(), transaction.state),
    };
    let next_state = match current_state.merged_with(update.state) {
        state if timed_out && !state.is_completed() => TransactionState::Failed,
        state => state,
    };
    let fields = rules::state_update(next_state, &update.changes);
    if next_state == current_state && !fields.has_field_changes() {
        let state = store.get_state(wallet_id, transaction_id.clone()).await?;
        return Ok(state.map(|state| GemTransactionStateResult { transaction_id, state }));
    }
    if !store.update_transaction(wallet_id, transaction_id.clone(), fields).await? {
        return Ok(None);
    }
    Ok(Some(GemTransactionStateResult {
        transaction_id,
        state: next_state,
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, TransactionChange, TransactionMetadata, TransactionSwapMetadata, TransactionType};
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryStore {
        states: Mutex<Vec<(TransactionId, TransactionState)>>,
        updates: Mutex<Vec<(TransactionId, GemTransactionStateUpdate)>>,
        renamed: Mutex<Vec<(TransactionId, TransactionId)>>,
        deleted: Mutex<Vec<TransactionId>>,
    }

    impl MemoryStore {
        fn with(states: Vec<(TransactionId, TransactionState)>) -> Arc<Self> {
            Arc::new(Self {
                states: Mutex::new(states),
                ..Default::default()
            })
        }
    }

    #[async_trait::async_trait]
    impl GemTransactionStateStore for MemoryStore {
        async fn get_state(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<TransactionState>, GemServiceError> {
            Ok(self.states.lock().unwrap().iter().find(|(id, _)| *id == transaction_id).map(|(_, state)| *state))
        }
        async fn rename_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId, new_transaction_id: TransactionId) -> Result<(), GemServiceError> {
            for entry in self.states.lock().unwrap().iter_mut().filter(|(id, _)| *id == transaction_id) {
                entry.0 = new_transaction_id.clone();
            }
            self.renamed.lock().unwrap().push((transaction_id, new_transaction_id));
            Ok(())
        }
        async fn delete_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<(), GemServiceError> {
            self.states.lock().unwrap().retain(|(id, _)| *id != transaction_id);
            self.deleted.lock().unwrap().push(transaction_id);
            Ok(())
        }
        async fn update_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId, update: GemTransactionStateUpdate) -> Result<bool, GemServiceError> {
            let mut states = self.states.lock().unwrap();
            let Some(entry) = states.iter_mut().find(|(id, _)| *id == transaction_id) else {
                return Ok(false);
            };
            entry.1 = update.state;
            self.updates.lock().unwrap().push((transaction_id, update));
            Ok(true)
        }
    }

    fn swap_metadata(to_value: &str) -> TransactionSwapMetadata {
        TransactionSwapMetadata {
            from_asset: AssetId::from_chain(Chain::Ethereum),
            from_value: "1000000000000000000".into(),
            to_asset: AssetId::from_chain(Chain::Bitcoin),
            to_value: to_value.into(),
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
            "1".into(),
            AssetId::from_chain(Chain::Ethereum),
            "1000000000000000000".into(),
            None,
            serde_json::to_value(swap_metadata("10000000000000000000")).ok(),
            created_at,
        )
    }

    fn update(state: TransactionState, changes: Vec<TransactionChange>) -> Result<TransactionUpdate, String> {
        Ok(TransactionUpdate::new(state, changes))
    }

    fn apply_update(
        store: &MemoryStore,
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
        let store = MemoryStore::with(vec![(id("hash"), TransactionState::Pending)]);

        let result = apply_update(
            &store,
            transaction("hash", TransactionState::Pending, now),
            update(
                TransactionState::InTransit,
                vec![TransactionChange::Metadata(TransactionMetadata::Swap(swap_metadata("9900000000000000000")))],
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
        let store = MemoryStore::with(vec![(id("hash"), TransactionState::Pending)]);

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
        let store = MemoryStore::with(vec![(id("hash"), TransactionState::Pending), (id("new-hash"), TransactionState::Confirmed)]);

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
                state: TransactionState::Confirmed
            }
        );
        assert_eq!(store.deleted.lock().unwrap().as_slice(), &[id("hash")]);
        assert!(store.renamed.lock().unwrap().is_empty());
        assert!(store.updates.lock().unwrap().is_empty());
    }

    #[test]
    fn test_in_transit_is_not_downgraded_to_pending() {
        let now = Utc::now();
        let store = MemoryStore::with(vec![(id("hash"), TransactionState::InTransit)]);

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
        let store = MemoryStore::with(vec![]);

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
        let store = MemoryStore::with(vec![(id("hash"), TransactionState::Pending)]);

        let fresh = apply_update(&store, transaction("hash", TransactionState::Pending, now), Err("offline".into()), now);
        assert!(matches!(fresh, Err(GemServiceError::Status { .. })));

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
}
