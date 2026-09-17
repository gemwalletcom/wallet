use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard};
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
    fn state(&self) -> MutexGuard<'_, TrackingState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn start(&self, transaction_id: &TransactionId) -> Option<TrackedTransactions<'_>> {
        let mut state = self.state();
        if state.polls.contains_key(transaction_id) {
            return None;
        }
        state.last_poll += 1;
        let poll = state.last_poll;
        state.polls.insert(transaction_id.clone(), poll);
        Some(TrackedTransactions { tracking: self, poll })
    }

    pub fn cancel(&self) {
        self.state().polls.clear();
    }
}

impl TrackedTransactions<'_> {
    fn is_tracking(&self) -> bool {
        self.tracking.state().polls.values().any(|poll| *poll == self.poll)
    }

    fn follow(&self, transaction_id: &TransactionId) {
        self.tracking.state().polls.insert(transaction_id.clone(), self.poll);
    }
}

impl Drop for TrackedTransactions<'_> {
    fn drop(&mut self) {
        self.tracking.state().polls.retain(|_, poll| *poll != self.poll);
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
    use primitives::{TransactionState, Wallet};
    use std::future::Future;
    use std::task::Context;

    use crate::services::transaction_state::model::GemPendingTransaction;
    use crate::services::transaction_state::testkit::{MemoryTransactionStateStore, TestTransactionUpdater};

    fn run(updater: &TestTransactionUpdater, store: &MemoryTransactionStateStore, tracking: &Tracking, transaction: Transaction) {
        futures::executor::block_on(poll(updater, store, tracking, JobConfiguration::mock(), WalletId::Multicoin("wallet".into()), transaction));
    }

    #[test]
    fn test_poll_follows_a_replaced_hash_and_stops_once_confirmed() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let replaced = Transaction {
            id: TransactionId::mock("new-hash"),
            ..Transaction::mock()
        };
        let store = MemoryTransactionStateStore {
            pending: Mutex::new(vec![GemPendingTransaction {
                wallet: Wallet::mock(),
                transaction: replaced.clone(),
            }]),
            ..Default::default()
        };
        let updater = TestTransactionUpdater {
            results: Mutex::new(vec![Ok(Some(GemTransactionStateResult::mock(replaced.id.clone(), TransactionState::Confirmed)))]),
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
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let updater = TestTransactionUpdater {
            results: Mutex::new(vec![Err(GemServiceError::Gateway { msg: "offline".to_string() }), Ok(None)]),
            ..Default::default()
        };
        let tracking = Tracking::default();

        run(&updater, &MemoryTransactionStateStore::default(), &tracking, pending.clone());

        assert_eq!(updater.requested.lock().unwrap().len(), 2);
        assert!(tracking.start(&pending.id).is_some());
    }

    #[test]
    fn test_poll_skips_a_transaction_that_is_already_tracked() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let updater = TestTransactionUpdater::default();
        let tracking = Tracking::default();
        let _owner = tracking.start(&pending.id).unwrap();

        run(&updater, &MemoryTransactionStateStore::default(), &tracking, pending);

        assert!(updater.requested.lock().unwrap().is_empty());
    }

    #[test]
    fn test_cancel_stops_the_running_poll_and_frees_its_transactions() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
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
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let updater = TestTransactionUpdater::default();
        let store = MemoryTransactionStateStore::default();
        let tracking = Tracking::default();

        {
            let mut polling = Box::pin(poll(
                &updater,
                &store,
                &tracking,
                JobConfiguration::mock(),
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
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
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
