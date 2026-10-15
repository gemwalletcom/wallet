use primitives::{Transaction, TransactionState, TransactionSwapMetadata, TransactionType, swap::SwapResult};
use storage::TransactionUpdate;

pub fn swap_result_metadata(transaction: &Transaction, metadata: Option<TransactionSwapMetadata>, preserve_referral_fee: bool) -> Option<serde_json::Value> {
    let existing_metadata = transaction.swap_metadata();
    let referral_fee = preserve_referral_fee.then(|| existing_metadata.clone().and_then(|metadata| metadata.referral_fee)).flatten();
    metadata
        .or(existing_metadata)
        .map(|metadata| match (preserve_referral_fee, metadata.referral_fee.is_some()) {
            (false, _) => metadata.with_referral_fee(None),
            (true, true) => metadata,
            (true, false) => metadata.with_referral_fee(referral_fee),
        })
        .and_then(|metadata| serde_json::to_value(metadata).ok())
}

pub fn transaction_with_swap_result(transaction: Transaction, result: SwapResult) -> Transaction {
    let Some(state) = result.status.transaction_state() else {
        return transaction;
    };
    let metadata = swap_result_metadata(&transaction, result.metadata, result.status.charges_referral_fee());
    transaction_with_swap_state(transaction, state, metadata)
}

pub fn transaction_with_swap_state(transaction: Transaction, state: TransactionState, swap_metadata: Option<serde_json::Value>) -> Transaction {
    let transaction_type = match state {
        TransactionState::Confirmed if swap_metadata.is_none() => transaction.transaction_type.clone(),
        TransactionState::Pending | TransactionState::Confirmed | TransactionState::InTransit | TransactionState::Failed | TransactionState::Reverted | TransactionState::Refunded => TransactionType::Swap,
    };
    Transaction {
        transaction_type,
        state,
        metadata: swap_metadata.or(transaction.metadata.clone()),
        ..transaction
    }
}

pub fn swap_state_updates(transaction: &Transaction, swap_metadata: Option<&serde_json::Value>) -> Vec<TransactionUpdate> {
    [TransactionUpdate::State(transaction.state), TransactionUpdate::Kind(transaction.transaction_type.clone())]
        .into_iter()
        .chain(swap_metadata.cloned().map(TransactionUpdate::Metadata))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{AssetId, Chain, SwapProvider, TransactionSwapReferralFee, swap::SwapStatus};

    #[test]
    fn test_transaction_with_swap_result() {
        let metadata = TransactionSwapMetadata::new(AssetId::from_chain(Chain::Solana), 1000u32.into(), AssetId::from_chain(Chain::Ethereum), 20u32.into(), SwapProvider::Mayan).with_referral_fee(Some(TransactionSwapReferralFee {
            asset_id: AssetId::from_chain(Chain::Solana),
            value: 5u32.into(),
        }));
        let completed = transaction_with_swap_result(
            Transaction::mock(),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: Some(metadata.clone()),
                eta_in_seconds: None,
            },
        );
        assert_eq!(completed.transaction_type, TransactionType::Swap);
        assert_eq!(completed.state, TransactionState::Confirmed);
        assert_eq!(completed.swap_metadata(), Some(metadata));

        let pending = transaction_with_swap_result(
            Transaction::mock(),
            SwapResult {
                status: SwapStatus::Pending,
                metadata: None,
                eta_in_seconds: None,
            },
        );
        assert_eq!(pending.transaction_type, TransactionType::Transfer);
        assert_eq!(pending.state, TransactionState::Confirmed);
        assert_eq!(pending.metadata, None);

        let refunded = transaction_with_swap_result(
            completed,
            SwapResult {
                status: SwapStatus::Refunded,
                metadata: None,
                eta_in_seconds: None,
            },
        );
        assert_eq!(refunded.state, TransactionState::Refunded);
        assert_eq!(refunded.swap_metadata().and_then(|metadata| metadata.referral_fee), None);

        let completed_without_metadata = transaction_with_swap_result(
            Transaction::mock(),
            SwapResult {
                status: SwapStatus::Completed,
                metadata: None,
                eta_in_seconds: None,
            },
        );
        assert_eq!(completed_without_metadata.transaction_type, TransactionType::Transfer);
        assert_eq!(completed_without_metadata.state, TransactionState::Confirmed);

        let failed_without_metadata = transaction_with_swap_result(
            Transaction::mock(),
            SwapResult {
                status: SwapStatus::Failed,
                metadata: None,
                eta_in_seconds: None,
            },
        );
        assert_eq!(failed_without_metadata.transaction_type, TransactionType::Swap);
        assert_eq!(failed_without_metadata.state, TransactionState::Failed);
    }
}
