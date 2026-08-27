use chrono::{DateTime, Utc};
use primitives::{AssetId, Chain, Transaction, TransactionChange, TransactionMetadata, TransactionState, TransactionType, swap_transaction_timeout};

use super::model::{GemTransactionPostProcessing, GemTransactionStateUpdate};
use crate::services::collections::unique;

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

pub fn post_processing(transaction: &Transaction, previous_state: TransactionState, state: TransactionState) -> Option<GemTransactionPostProcessing> {
    let entered_transit = previous_state == TransactionState::Pending && state == TransactionState::InTransit;
    if !state.is_completed() && !entered_transit {
        return None;
    }
    let balance_asset_ids = transaction.associated_asset_ids();
    if !state.is_completed() {
        return Some(GemTransactionPostProcessing {
            balance_asset_ids,
            ..Default::default()
        });
    }
    let mut processing = GemTransactionPostProcessing {
        balance_asset_ids,
        ..Default::default()
    };
    match transaction.transaction_type {
        TransactionType::StakeDelegate
        | TransactionType::StakeUndelegate
        | TransactionType::StakeRewards
        | TransactionType::StakeRedelegate
        | TransactionType::StakeWithdraw
        | TransactionType::StakeFreeze
        | TransactionType::StakeUnfreeze => {
            processing.stake_chains = transaction.asset_ids().into_iter().map(|asset_id| asset_id.chain).collect();
            processing.stake_chains.dedup();
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

pub fn state_update(state: TransactionState, changes: &[TransactionChange]) -> Result<GemTransactionStateUpdate, serde_json::Error> {
    let mut update = GemTransactionStateUpdate::new(state);
    for change in changes {
        match change {
            TransactionChange::NetworkFee(fee) => update.fee = Some(fee.to_string()),
            TransactionChange::BlockNumber(number) => update.block_number = Some(number.clone()),
            TransactionChange::Metadata(metadata) => update.metadata = Some(metadata_json(metadata)?),
            TransactionChange::ConfirmationEtaSeconds(seconds) => update.confirmation_eta_seconds = Some(*seconds),
            TransactionChange::HashChange { .. } => {}
        }
    }
    Ok(update)
}

fn metadata_json(metadata: &TransactionMetadata) -> Result<String, serde_json::Error> {
    match metadata {
        TransactionMetadata::Swap(swap) => serde_json::to_string(swap),
        TransactionMetadata::Perpetual(perpetual) => serde_json::to_string(perpetual),
    }
}

pub fn assets_to_enable(transactions: &[Transaction]) -> Vec<AssetId> {
    unique(transactions.iter().flat_map(Transaction::asset_ids).filter(|asset_id| asset_id.chain != Chain::HyperCore))
}
