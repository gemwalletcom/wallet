use chrono::{DateTime, Utc};
use primitives::{Chain, Transaction, TransactionChange, TransactionMetadata, TransactionState, swap_transaction_timeout};

use super::model::GemTransactionStateUpdate;

pub fn destination_chain(transaction: &Transaction) -> Option<Chain> {
    (transaction.state == TransactionState::InTransit)
        .then(|| transaction.swap_metadata().map(|metadata| metadata.to_asset.chain))
        .flatten()
}

pub fn has_timed_out(transaction: &Transaction, now: DateTime<Utc>) -> bool {
    if transaction.state.is_completed() {
        return false;
    }
    let chain = transaction.asset_id.chain;
    let timeout_ms = swap_transaction_timeout(chain, destination_chain(transaction).unwrap_or(chain));
    (now - transaction.created_at).num_milliseconds() > timeout_ms as i64
}

pub fn new_hash(changes: &[TransactionChange]) -> Option<String> {
    changes.iter().find_map(|change| match change {
        TransactionChange::HashChange { new, .. } => Some(new.clone()),
        _ => None,
    })
}

pub fn state_update(state: TransactionState, changes: &[TransactionChange]) -> GemTransactionStateUpdate {
    changes.iter().fold(GemTransactionStateUpdate::new(state), |mut update, change| {
        match change {
            TransactionChange::NetworkFee(fee) => update.fee = Some(fee.to_string()),
            TransactionChange::BlockNumber(number) => update.block_number = Some(number.clone()),
            TransactionChange::Metadata(metadata) => update.metadata = metadata_json(metadata),
            TransactionChange::ConfirmationEtaSeconds(seconds) => update.confirmation_eta_seconds = Some(*seconds),
            TransactionChange::HashChange { .. } => {}
        }
        update
    })
}

fn metadata_json(metadata: &TransactionMetadata) -> Option<String> {
    match metadata {
        TransactionMetadata::Swap(swap) => serde_json::to_string(swap).ok(),
        TransactionMetadata::Perpetual(perpetual) => serde_json::to_string(perpetual).ok(),
    }
}
