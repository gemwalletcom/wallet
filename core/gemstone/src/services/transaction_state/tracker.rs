use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use primitives::{JobConfiguration, Transaction, TransactionId, WalletId};

use crate::services::clock::sleep;
use crate::services::error::GemServiceError;

use super::model::GemTransactionStateResult;
use super::store::GemTransactionStateStore;

#[async_trait]
pub trait GemTransactionUpdater: Send + Sync {
    async fn update(&self, wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError>;
}

#[derive(Default)]
pub struct Tracking {
    state: Mutex<TrackingState>,
}

#[derive(Default)]
struct TrackingState {
    last_poll: u64,
    polls: HashMap<TransactionId, u64>,
}

pub struct TrackedTransactions<'a> {
    tracking: &'a Tracking,
    poll: u64,
}

impl Tracking {
    pub fn start(&self, transaction_id: &TransactionId) -> Option<TrackedTransactions<'_>> {
        let mut state = self.state.lock().unwrap();
        if state.polls.contains_key(transaction_id) {
            return None;
        }
        state.last_poll += 1;
        let poll = state.last_poll;
        state.polls.insert(transaction_id.clone(), poll);
        Some(TrackedTransactions { tracking: self, poll })
    }

    pub fn cancel(&self) {
        self.state.lock().unwrap().polls.clear();
    }
}

impl TrackedTransactions<'_> {
    fn is_tracking(&self) -> bool {
        self.tracking.state.lock().unwrap().polls.values().any(|poll| *poll == self.poll)
    }

    fn follow(&self, transaction_id: &TransactionId) {
        self.tracking.state.lock().unwrap().polls.insert(transaction_id.clone(), self.poll);
    }
}

impl Drop for TrackedTransactions<'_> {
    fn drop(&mut self) {
        self.tracking.state.lock().unwrap().polls.retain(|_, poll| *poll != self.poll);
    }
}

pub async fn poll(
    updater: &dyn GemTransactionUpdater,
    store: &dyn GemTransactionStateStore,
    tracking: &Tracking,
    configuration: JobConfiguration,
    wallet_id: WalletId,
    transaction: Transaction,
) {
    let Some(tracked) = tracking.start(&transaction.id) else {
        return;
    };
    let mut current = transaction;
    let mut interval = configuration.initial_interval_ms;

    loop {
        sleep(Duration::from_millis(u64::from(interval))).await;
        interval = configuration.next_interval_ms(interval);
        if !tracked.is_tracking() {
            break;
        }
        let result = match updater.update(wallet_id.clone(), current.clone()).await {
            Ok(Some(result)) => result,
            Ok(None) => break,
            Err(_) => continue,
        };
        if result.transaction_id != current.id {
            tracked.follow(&result.transaction_id);
        }
        let stored = store.get_transaction(wallet_id.clone(), result.transaction_id.clone()).await;
        let Ok(Some(pending)) = stored else {
            break;
        };
        current = pending.transaction;
        if current.state.is_completed() {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, TransactionState, Wallet};
    use std::future::Future;
    use std::task::Context;

    use crate::services::transaction_state::model::{GemPendingTransaction, GemTransactionStateUpdate};

    #[derive(Default)]
    struct StubStore {
        transactions: Mutex<Vec<Transaction>>,
    }

    #[async_trait]
    impl GemTransactionStateStore for StubStore {
        async fn get_pending_transactions(&self) -> Result<Vec<GemPendingTransaction>, GemServiceError> {
            Ok(Vec::new())
        }

        async fn get_transaction(&self, _wallet_id: WalletId, transaction_id: TransactionId) -> Result<Option<GemPendingTransaction>, GemServiceError> {
            let transactions = self.transactions.lock().unwrap();
            Ok(transactions
                .iter()
                .find(|transaction| transaction.id == transaction_id)
                .map(|transaction| GemPendingTransaction {
                    wallet: wallet(),
                    transaction: transaction.clone(),
                }))
        }

        async fn add_transactions(&self, _wallet_id: WalletId, _transactions: Vec<Transaction>) -> Result<(), GemServiceError> {
            Ok(())
        }

        async fn get_state(&self, _wallet_id: WalletId, _transaction_id: TransactionId) -> Result<Option<TransactionState>, GemServiceError> {
            Ok(None)
        }

        async fn rename_transaction(&self, _wallet_id: WalletId, _transaction_id: TransactionId, _new_transaction_id: TransactionId) -> Result<(), GemServiceError> {
            Ok(())
        }

        async fn delete_transaction(&self, _wallet_id: WalletId, _transaction_id: TransactionId) -> Result<(), GemServiceError> {
            Ok(())
        }

        async fn update_transaction(&self, _wallet_id: WalletId, _transaction_id: TransactionId, _update: GemTransactionStateUpdate) -> Result<bool, GemServiceError> {
            Ok(true)
        }
    }

    #[derive(Default)]
    struct StubUpdater {
        results: Mutex<Vec<Result<Option<GemTransactionStateResult>, GemServiceError>>>,
        requested: Mutex<Vec<TransactionId>>,
    }

    #[async_trait]
    impl GemTransactionUpdater for StubUpdater {
        async fn update(&self, _wallet_id: WalletId, transaction: Transaction) -> Result<Option<GemTransactionStateResult>, GemServiceError> {
            self.requested.lock().unwrap().push(transaction.id.clone());
            let mut results = self.results.lock().unwrap();
            if results.is_empty() {
                return Ok(None);
            }
            results.remove(0)
        }
    }

    fn wallet() -> Wallet {
        Wallet {
            id: WalletId::Multicoin("wallet".into()),
            ..Wallet::mock()
        }
    }

    fn configuration() -> JobConfiguration {
        JobConfiguration {
            initial_interval_ms: 1,
            max_interval_ms: 1,
            step_factor: 1.0,
        }
    }

    fn transaction(hash: &str, state: TransactionState) -> Transaction {
        let mut transaction = Transaction::mock();
        transaction.id = TransactionId::new(Chain::Ethereum, hash.to_string());
        transaction.asset_id = AssetId::from_chain(Chain::Ethereum);
        transaction.state = state;
        transaction
    }

    fn state_result(transaction_id: &TransactionId, state: TransactionState) -> GemTransactionStateResult {
        GemTransactionStateResult {
            transaction_id: transaction_id.clone(),
            state,
            failures: Vec::new(),
        }
    }

    fn run(updater: &StubUpdater, store: &StubStore, tracking: &Tracking, transaction: Transaction) {
        futures::executor::block_on(poll(updater, store, tracking, configuration(), WalletId::Multicoin("wallet".into()), transaction));
    }

    #[test]
    fn test_poll_follows_a_replaced_hash_and_stops_once_confirmed() {
        let pending = transaction("hash", TransactionState::Pending);
        let replaced = transaction("new-hash", TransactionState::Confirmed);
        let store = StubStore {
            transactions: Mutex::new(vec![replaced.clone()]),
        };
        let updater = StubUpdater {
            results: Mutex::new(vec![Ok(Some(state_result(&replaced.id, TransactionState::Confirmed)))]),
            ..Default::default()
        };
        let tracking = Tracking::default();

        run(&updater, &store, &tracking, pending.clone());

        assert_eq!(*updater.requested.lock().unwrap(), vec![pending.id.clone()]);
        assert!(tracking.start(&pending.id).is_some());
        assert!(tracking.start(&replaced.id).is_some());
    }

    #[test]
    fn test_poll_retries_after_an_update_error_and_stops_when_the_transaction_is_gone() {
        let pending = transaction("hash", TransactionState::Pending);
        let updater = StubUpdater {
            results: Mutex::new(vec![Err(GemServiceError::Gateway { msg: "offline".to_string() }), Ok(None)]),
            ..Default::default()
        };
        let tracking = Tracking::default();

        run(&updater, &StubStore::default(), &tracking, pending.clone());

        assert_eq!(updater.requested.lock().unwrap().len(), 2);
        assert!(tracking.start(&pending.id).is_some());
    }

    #[test]
    fn test_poll_skips_a_transaction_that_is_already_tracked() {
        let pending = transaction("hash", TransactionState::Pending);
        let updater = StubUpdater::default();
        let tracking = Tracking::default();
        let _owner = tracking.start(&pending.id).unwrap();

        run(&updater, &StubStore::default(), &tracking, pending);

        assert!(updater.requested.lock().unwrap().is_empty());
    }

    #[test]
    fn test_cancel_stops_the_running_poll_and_frees_its_transactions() {
        let pending = transaction("hash", TransactionState::Pending);
        let tracking = Tracking::default();
        let tracked = tracking.start(&pending.id).unwrap();

        assert!(tracking.start(&pending.id).is_none());
        assert!(tracked.is_tracking());

        tracking.cancel();

        assert!(!tracked.is_tracking());
        assert!(tracking.start(&pending.id).is_some());
    }

    #[test]
    fn test_a_poll_dropped_mid_flight_releases_the_transaction() {
        let pending = transaction("hash", TransactionState::Pending);
        let updater = StubUpdater::default();
        let store = StubStore::default();
        let tracking = Tracking::default();

        {
            let mut polling = Box::pin(poll(
                &updater,
                &store,
                &tracking,
                configuration(),
                WalletId::Multicoin("wallet".into()),
                pending.clone(),
            ));
            let waker = futures::task::noop_waker();
            assert!(polling.as_mut().poll(&mut Context::from_waker(&waker)).is_pending());
            assert!(tracking.start(&pending.id).is_none(), "the poll owns the transaction while it runs");
        }

        assert!(
            tracking.start(&pending.id).is_some(),
            "a poll dropped at its first sleep must free the transaction, or it is never tracked again"
        );
    }

    #[test]
    fn test_a_poll_dropped_after_a_restart_leaves_the_new_owner_alone() {
        let pending = transaction("hash", TransactionState::Pending);
        let tracking = Tracking::default();
        let stopped = tracking.start(&pending.id).unwrap();

        tracking.cancel();
        let restarted = tracking.start(&pending.id).unwrap();
        drop(stopped);

        assert!(tracking.start(&pending.id).is_none(), "the restarted poll still owns the transaction");

        drop(restarted);
        assert!(tracking.start(&pending.id).is_some());
    }
}
