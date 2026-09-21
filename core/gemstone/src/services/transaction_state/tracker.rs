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

type PollKey = (WalletId, TransactionId);

#[derive(Default)]
struct TrackingState {
    last_poll: u64,
    polls: HashMap<PollKey, u64>,
}

impl TrackingState {
    fn owns(&self, poll: u64) -> bool {
        self.polls.values().any(|owner| *owner == poll)
    }
}

pub struct TrackedTransactions<'a> {
    tracking: &'a Tracking,
    poll: u64,
}

impl Tracking {
    fn state(&self) -> MutexGuard<'_, TrackingState> {
        self.state.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn start(&self, wallet_id: &WalletId, transaction_id: &TransactionId) -> Option<TrackedTransactions<'_>> {
        let mut state = self.state();
        let key = (wallet_id.clone(), transaction_id.clone());
        if state.polls.contains_key(&key) {
            return None;
        }
        state.last_poll += 1;
        let poll = state.last_poll;
        state.polls.insert(key, poll);
        Some(TrackedTransactions { tracking: self, poll })
    }

    pub fn cancel(&self) {
        self.state().polls.clear();
    }
}

impl TrackedTransactions<'_> {
    fn is_tracking(&self) -> bool {
        self.tracking.state().owns(self.poll)
    }

    fn follow(&self, wallet_id: &WalletId, transaction_id: &TransactionId) -> bool {
        let mut state = self.tracking.state();
        if !state.owns(self.poll) {
            return false;
        }
        let key = (wallet_id.clone(), transaction_id.clone());
        if state.polls.get(&key).is_some_and(|owner| *owner != self.poll) {
            return false;
        }
        state.polls.insert(key, self.poll);
        true
    }
}

impl Drop for TrackedTransactions<'_> {
    fn drop(&mut self) {
        self.tracking.state().polls.retain(|_, poll| *poll != self.poll);
    }
}

pub async fn poll(updater: &dyn GemTransactionUpdater, store: &dyn GemTransactionStateStore, tracking: &Tracking, configuration: JobConfiguration, wallet_id: WalletId, transaction: Transaction) {
    let Some(tracked) = tracking.start(&wallet_id, &transaction.id) else {
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
        if result.transaction_id != current.id && !tracked.follow(&wallet_id, &result.transaction_id) {
            break;
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

    fn wallet_id(name: &str) -> WalletId {
        WalletId::Multicoin(name.into())
    }

    fn run(updater: &TestTransactionUpdater, store: &MemoryTransactionStateStore, tracking: &Tracking, transaction: Transaction) {
        run_for(&wallet_id("wallet"), updater, store, tracking, transaction);
    }

    fn run_for(wallet: &WalletId, updater: &TestTransactionUpdater, store: &MemoryTransactionStateStore, tracking: &Tracking, transaction: Transaction) {
        futures::executor::block_on(poll(updater, store, tracking, JobConfiguration::mock(), wallet.clone(), transaction));
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
        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_some());
        assert!(tracking.start(&wallet_id("wallet"), &replaced.id).is_some());
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
        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_some());
    }

    #[test]
    fn test_poll_skips_a_transaction_that_is_already_tracked() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let updater = TestTransactionUpdater::default();
        let tracking = Tracking::default();
        let _owner = tracking.start(&wallet_id("wallet"), &pending.id).unwrap();

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
        let tracked = tracking.start(&wallet_id("wallet"), &pending.id).unwrap();

        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_none());
        assert!(tracked.is_tracking());

        tracking.cancel();

        assert!(!tracked.is_tracking());
        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_some());
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
            let mut polling = Box::pin(poll(&updater, &store, &tracking, JobConfiguration::mock(), wallet_id("wallet"), pending.clone()));
            let waker = futures::task::noop_waker();
            assert!(polling.as_mut().poll(&mut Context::from_waker(&waker)).is_pending());
            assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_none(), "the poll owns the transaction while it runs");
        }

        assert!(
            tracking.start(&wallet_id("wallet"), &pending.id).is_some(),
            "a poll dropped at its first sleep must free the transaction, or it is never tracked again"
        );
    }

    #[test]
    fn test_the_same_transaction_is_polled_once_for_every_wallet_that_holds_it() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let tracking = Tracking::default();
        let second = TestTransactionUpdater::default();
        let polling_first_wallet = tracking.start(&wallet_id("first"), &pending.id).unwrap();

        run_for(&wallet_id("second"), &second, &MemoryTransactionStateStore::default(), &tracking, pending.clone());

        assert_eq!(*second.requested.lock().unwrap(), vec![pending.id.clone()], "a wallet is not skipped because another wallet is polling the same transaction");
        assert!(tracking.start(&wallet_id("first"), &pending.id).is_none(), "one wallet still polls a transaction once");
        drop(polling_first_wallet);
    }

    #[test]
    fn test_a_cancelled_poll_cannot_take_a_replacement_hash_back() {
        let wallet = wallet_id("wallet");
        let pending = TransactionId::mock("hash");
        let replacement = TransactionId::mock("new-hash");
        let tracking = Tracking::default();
        let stopped = tracking.start(&wallet, &pending).unwrap();

        tracking.cancel();
        let restarted = tracking.start(&wallet, &pending).unwrap();

        assert!(!stopped.follow(&wallet, &replacement), "a cancelled poll does not claim the hash it was told about");
        assert!(tracking.start(&wallet, &replacement).is_some(), "the replacement was never claimed");

        assert!(restarted.follow(&wallet, &replacement));
        assert!(!stopped.follow(&wallet, &replacement), "the live owner keeps the hash");
        assert!(restarted.is_tracking());
    }

    #[test]
    fn test_a_poll_dropped_after_a_restart_leaves_the_new_owner_alone() {
        let pending = Transaction {
            state: TransactionState::Pending,
            ..Transaction::mock()
        };
        let tracking = Tracking::default();
        let stopped = tracking.start(&wallet_id("wallet"), &pending.id).unwrap();

        tracking.cancel();
        let restarted = tracking.start(&wallet_id("wallet"), &pending.id).unwrap();
        drop(stopped);

        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_none(), "the restarted poll still owns the transaction");

        drop(restarted);
        assert!(tracking.start(&wallet_id("wallet"), &pending.id).is_some());
    }
}
