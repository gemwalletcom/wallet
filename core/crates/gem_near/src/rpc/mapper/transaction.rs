use std::error::Error;

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::transaction_metadata_types::{TransactionAssetTransfer, TransactionAssetTransfersMetadata};
use primitives::{Chain, Transaction, TransactionState, TransactionType};

use crate::models::{BroadcastTransaction, Outcome, TransactionAction};

use super::fungible_token::map_fungible_token_transfers;

pub(in crate::rpc) fn map_transaction(
    transaction: BroadcastTransaction,
    receipts: Vec<Outcome>,
    block_height: u64,
    block_timestamp: u64,
    state: TransactionState,
    fee: BigUint,
) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let created_at = DateTime::<Utc>::from_timestamp_nanos(i64::try_from(block_timestamp)?);
    let asset_transfers = map_fungible_token_transfers(&receipts)?;
    let (transfer, transaction_type, metadata) = if let Some(transfer) = asset_transfers.first().cloned() {
        let metadata = if asset_transfers.len() > 1 {
            Some(serde_json::to_value(TransactionAssetTransfersMetadata { asset_transfers })?)
        } else {
            None
        };
        (transfer, TransactionType::Transfer, metadata)
    } else {
        let (from, to, actions) = transaction
            .actions
            .iter()
            .find_map(|action| {
                if let TransactionAction::Delegate { delegate } = action {
                    Some((
                        delegate.delegate_action.sender_id.as_str(),
                        delegate.delegate_action.receiver_id.as_str(),
                        delegate.delegate_action.actions.as_slice(),
                    ))
                } else {
                    None
                }
            })
            .unwrap_or((&transaction.signer_id, &transaction.receiver_id, &transaction.actions));
        let mut transfer_deposits = actions
            .iter()
            .filter_map(|action| match action {
                TransactionAction::Transfer { transfer } => Some(&transfer.deposit),
                _ => None,
            })
            .peekable();
        let (transaction_type, value) = if transfer_deposits.peek().is_some() {
            (
                TransactionType::Transfer,
                transfer_deposits.map(|deposit| deposit.parse::<BigUint>()).sum::<Result<BigUint, _>>()?,
            )
        } else {
            (
                TransactionType::SmartContractCall,
                actions
                    .iter()
                    .filter_map(|action| match action {
                        TransactionAction::FunctionCall { function_call } => Some(function_call),
                        _ => None,
                    })
                    .map(|function_call| function_call.deposit.parse::<BigUint>())
                    .sum::<Result<BigUint, _>>()?,
            )
        };
        (
            TransactionAssetTransfer {
                asset_id: Chain::Near.as_asset_id(),
                from: from.to_string(),
                to: to.to_string(),
                value,
            },
            transaction_type,
            None,
        )
    };
    let mut mapped = Transaction::new(
        transaction.hash,
        transfer.asset_id,
        transfer.from,
        transfer.to,
        None,
        transaction_type,
        state,
        fee.to_string(),
        Chain::Near.as_asset_id(),
        transfer.value.to_string(),
        None,
        metadata,
        created_at,
    );
    mapped.block_number = Some(block_height.to_string());
    Ok(mapped)
}
