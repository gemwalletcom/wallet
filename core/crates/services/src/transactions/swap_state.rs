use primitives::{Transaction, TransactionState, TransactionSwapMetadata, TransactionType, swap::SwapResult};
use storage::TransactionUpdate;

pub fn swap_result_metadata(transaction: &Transaction, metadata: Option<TransactionSwapMetadata>, preserve_referral_fee: bool) -> Option<serde_json::Value> {
    let referral_fee = preserve_referral_fee.then(|| transaction.swap_metadata().and_then(|metadata| metadata.referral_fee)).flatten();
    metadata
        .map(|metadata| match metadata.referral_fee {
            Some(_) => metadata,
            None => metadata.with_referral_fee(referral_fee),
        })
        .and_then(|metadata| serde_json::to_value(metadata).ok())
}

pub fn transaction_with_swap_result(transaction: Transaction, result: SwapResult) -> Transaction {
    let Some(state) = result.status.transaction_state() else {
        return transaction;
    };
    let metadata = swap_result_metadata(&transaction, result.metadata, result.status.charges_referral_fee()).or(transaction.metadata.clone());
    Transaction {
        transaction_type: TransactionType::Swap,
        state,
        metadata,
        ..transaction
    }
}

pub fn swap_state_updates(state: TransactionState, metadata: Option<&serde_json::Value>) -> Vec<TransactionUpdate> {
    [TransactionUpdate::State(state), TransactionUpdate::Kind(TransactionType::Swap)]
        .into_iter()
        .chain(metadata.cloned().map(TransactionUpdate::Metadata))
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
                metadata: Some(TransactionSwapMetadata::new(
                    AssetId::from_chain(Chain::Solana),
                    1000u32.into(),
                    AssetId::from_chain(Chain::Ethereum),
                    20u32.into(),
                    SwapProvider::Mayan,
                )),
                eta_in_seconds: None,
            },
        );
        assert_eq!(refunded.state, TransactionState::Refunded);
        assert_eq!(refunded.swap_metadata().and_then(|metadata| metadata.referral_fee), None);
    }
}
