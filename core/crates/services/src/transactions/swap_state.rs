use primitives::{Transaction, TransactionState, TransactionSwapMetadata, TransactionType};
use storage::TransactionUpdate;

pub fn swap_result_metadata(transaction: &Transaction, metadata: Option<TransactionSwapMetadata>) -> Option<serde_json::Value> {
    let referral_fee = transaction.swap_metadata().and_then(|metadata| metadata.referral_fee);
    metadata
        .map(|metadata| match metadata.referral_fee {
            Some(_) => metadata,
            None => metadata.with_referral_fee(referral_fee),
        })
        .and_then(|metadata| serde_json::to_value(metadata).ok())
}

pub fn swap_state_updates(state: TransactionState, metadata: Option<&serde_json::Value>) -> Vec<TransactionUpdate> {
    [TransactionUpdate::State(state), TransactionUpdate::Kind(TransactionType::Swap)]
        .into_iter()
        .chain(metadata.cloned().map(TransactionUpdate::Metadata))
        .collect()
}
