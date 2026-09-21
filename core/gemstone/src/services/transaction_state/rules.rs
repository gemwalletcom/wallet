use chrono::{DateTime, Utc};
use primitives::{AssetId, Chain, PaymentLink, Transaction, TransactionChange, TransactionMetadata, TransactionState, TransactionType, swap_transaction_timeout};

use super::model::{GemTransactionStateUpdate, TransactionPostProcessing};
use crate::payment::payment_record_hash;
use crate::services::collections::unique;

pub fn destination_chain(transaction: &Transaction) -> Option<Chain> {
    (transaction.state == TransactionState::InTransit).then(|| transaction.swap_metadata().map(|metadata| metadata.to_asset.chain)).flatten()
}

pub fn has_timed_out(transaction: &Transaction, now: DateTime<Utc>) -> bool {
    if transaction.state.is_completed() {
        return false;
    }
    let chain = transaction.asset_id.chain;
    let timeout_ms = swap_transaction_timeout(chain, destination_chain(transaction).unwrap_or(chain));
    (now - transaction.created_at).num_milliseconds() > timeout_ms as i64
}

pub fn post_processing(transaction: &Transaction, previous_state: TransactionState, state: TransactionState) -> Option<TransactionPostProcessing> {
    let entered_transit = previous_state == TransactionState::Pending && state == TransactionState::InTransit;
    if !state.is_completed() && !entered_transit {
        return None;
    }
    let balance_asset_ids = transaction.associated_asset_ids();
    if !state.is_completed() {
        return Some(TransactionPostProcessing { balance_asset_ids, ..Default::default() });
    }
    let mut processing = TransactionPostProcessing { balance_asset_ids, ..Default::default() };
    match transaction.transaction_type {
        TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeRedelegate
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze => {
            processing.stake_chains = unique(transaction.asset_ids().into_iter().map(|asset_id| asset_id.chain));
        }
        TransactionType::EarnDeposit | TransactionType::EarnWithdraw => processing.earn_asset_ids = transaction.asset_ids(),
        TransactionType::TransferNFT => processing.sync_nfts = true,
        _ => {}
    }
    Some(processing)
}

pub fn new_hash(changes: &[TransactionChange]) -> Option<String> {
    changes.iter().find_map(|change| match change {
        TransactionChange::HashChange { new, .. } => Some(new.clone()),
        _ => None,
    })
}

pub fn state_update(state: TransactionState, changes: &[TransactionChange], transaction: &Transaction) -> Result<GemTransactionStateUpdate, serde_json::Error> {
    let mut update = GemTransactionStateUpdate::new(state);
    for change in changes {
        match change {
            TransactionChange::NetworkFee(fee) => update.fee = Some(fee.clone()),
            TransactionChange::BlockNumber(number) => update.block_number = Some(number.clone()),
            TransactionChange::Metadata(metadata) => update.metadata = Some(metadata_json(metadata)?),
            TransactionChange::ConfirmationEtaSeconds(seconds) => update.confirmation_eta_seconds = Some(*seconds),
            TransactionChange::HashChange { .. } => {}
        }
    }
    if let Some(metadata) = &update.metadata {
        let mut updated = transaction.clone();
        updated.metadata = Some(serde_json::from_str(metadata)?);
        update.asset_ids = Some(updated.asset_ids());
    }
    Ok(update)
}

fn metadata_json(metadata: &TransactionMetadata) -> Result<String, serde_json::Error> {
    match metadata {
        TransactionMetadata::Swap(swap) => serde_json::to_string(swap),
        TransactionMetadata::Perpetual(perpetual) => serde_json::to_string(perpetual),
        TransactionMetadata::Payment(payment) => serde_json::to_string(payment),
    }
}

pub fn payment_link(transaction: &Transaction) -> Option<PaymentLink> {
    let link = transaction.payment_metadata()?.link;
    (payment_record_hash(&link).as_deref() == Some(transaction.hash())).then_some(link)
}

pub fn assets_to_enable(transactions: &[Transaction]) -> Vec<AssetId> {
    unique(transactions.iter().flat_map(Transaction::asset_ids).filter(|asset_id| asset_id.chain != Chain::HyperCore))
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{PaymentMerchant, TransactionId, TransactionPaymentMetadata, TransactionSwapMetadata};

    #[test]
    fn test_payment_link_only_while_the_record_carries_the_payment_id() {
        let mut transaction = Transaction::mock();
        assert_eq!(payment_link(&transaction), None);

        transaction.metadata = Some(serde_json::json!({"fromAsset": "ethereum", "fromValue": "1", "toAsset": "bitcoin", "toValue": "1", "provider": null}));
        assert_eq!(payment_link(&transaction), None);

        let link = PaymentLink::WalletConnectPay { payment_id: "pay_1".to_string() };
        transaction.metadata = Some(
            serde_json::to_value(TransactionPaymentMetadata {
                link: link.clone(),
                merchant: PaymentMerchant::mock(),
            })
            .unwrap(),
        );
        assert_eq!(payment_link(&transaction), None, "a payment with a chain hash is tracked on chain");

        transaction.id = TransactionId::new(Chain::Ethereum, "pay_1".to_string());
        assert_eq!(payment_link(&transaction), Some(link));
    }

    #[test]
    fn test_the_state_update_carries_the_asset_ids_the_new_metadata_moves() {
        let mut transaction = Transaction::mock();
        transaction.transaction_type = TransactionType::Swap;
        transaction.asset_id = AssetId::from_chain(Chain::Ethereum);
        let swap = TransactionSwapMetadata {
            from_asset: AssetId::from_chain(Chain::Ethereum),
            from_value: 100u32.into(),
            to_asset: AssetId::from_chain(Chain::Solana),
            to_value: 200u32.into(),
            provider: None,
        };

        let update = state_update(TransactionState::Confirmed, &[TransactionChange::Metadata(TransactionMetadata::Swap(swap))], &transaction).unwrap();

        let mut asset_ids = update.asset_ids.expect("the swap moved two assets");
        asset_ids.sort_by_key(|asset_id| asset_id.to_string());
        assert_eq!(asset_ids, vec![AssetId::from_chain(Chain::Ethereum), AssetId::from_chain(Chain::Solana)]);
        assert_eq!(
            state_update(TransactionState::Confirmed, &[TransactionChange::BlockNumber("1".to_string())], &transaction).unwrap().asset_ids,
            None,
            "an update that leaves the metadata alone moves no assets between rows"
        );
    }

    #[test]
    fn test_post_processing_stake_chains_are_unique() {
        let mut transaction = Transaction::mock();
        transaction.transaction_type = TransactionType::StakeDelegate;
        transaction.asset_id = AssetId::from_chain(Chain::Ethereum);
        transaction.metadata = Some(serde_json::json!({
            "assetTransfers": [
                { "assetId": "solana", "from": "a", "to": "b", "value": "1" },
                { "assetId": "ethereum_0xdAC17F958D2ee523a2206206994597C13D831ec7", "from": "a", "to": "b", "value": "1" }
            ]
        }));

        let processing = post_processing(&transaction, TransactionState::Pending, TransactionState::Confirmed).unwrap();

        let mut chains = processing.stake_chains.clone();
        chains.sort();
        assert_eq!(processing.stake_chains.len(), 2);
        assert_eq!(chains, vec![Chain::Ethereum, Chain::Solana]);
    }
}
