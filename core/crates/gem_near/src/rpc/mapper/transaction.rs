use std::error::Error;

use num_bigint::BigUint;
use primitives::transaction_metadata_types::{TransactionAssetTransfer, TransactionAssetTransfersMetadata};
use primitives::{Chain, DateTime, Transaction, TransactionState, TransactionType};

use crate::constants::FUNGIBLE_TOKEN_TRANSFER_METHODS;
use crate::models::{BroadcastResult, BroadcastTransaction, ExecutionStatus, Outcome, TransactionAction};

use super::fungible_token::map_fungible_token_transfers;

pub(in crate::rpc) fn is_transfer(action: &TransactionAction) -> bool {
    match action {
        TransactionAction::Transfer { .. } => true,
        TransactionAction::FunctionCall { function_call } => FUNGIBLE_TOKEN_TRANSFER_METHODS.contains(&function_call.method_name.as_str()),
        TransactionAction::Delegate { delegate } => delegate.delegate_action.actions.iter().any(is_transfer),
        TransactionAction::Other(_) => false,
    }
}

pub(in crate::rpc) fn map_execution_state<'a>(statuses: impl IntoIterator<Item = &'a ExecutionStatus>) -> TransactionState {
    let mut pending = false;
    for status in statuses {
        match status {
            ExecutionStatus::Failure(_) => return TransactionState::Failed,
            ExecutionStatus::NotStarted | ExecutionStatus::Started => pending = true,
            ExecutionStatus::SuccessReceiptId(_) | ExecutionStatus::SuccessValue(_) => {}
        }
    }
    if pending { TransactionState::Pending } else { TransactionState::Confirmed }
}

pub(in crate::rpc) fn map_transaction_result(result: BroadcastResult, block_height: u64, created_at: DateTime) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let state = if result.is_executed() {
        map_execution_state(std::iter::once(&result.transaction_outcome.outcome.status).chain(result.receipts_outcome.iter().map(|receipt| &receipt.outcome.status)))
    } else {
        TransactionState::Pending
    };
    let fee = result.fee();
    let receipts = result.receipts_outcome.into_iter().map(|receipt| receipt.outcome).collect();
    map_transaction(result.transaction, receipts, block_height, created_at, state, fee)
}

pub(in crate::rpc) fn map_transaction(transaction: BroadcastTransaction, receipts: Vec<Outcome>, block_height: u64, created_at: DateTime, state: TransactionState, fee: BigUint) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
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
                    Some((delegate.delegate_action.sender_id.as_str(), delegate.delegate_action.receiver_id.as_str(), delegate.delegate_action.actions.as_slice()))
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
            (TransactionType::Transfer, transfer_deposits.map(|deposit| deposit.parse::<BigUint>()).sum::<Result<BigUint, _>>()?)
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
        fee,
        Chain::Near.as_asset_id(),
        transfer.value,
        None,
        metadata,
        created_at,
    );
    mapped.block_number = Some(block_height.to_string());
    Ok(mapped)
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn action(value: Value) -> TransactionAction {
        serde_json::from_value(value).unwrap()
    }

    #[test]
    fn test_is_transfer() {
        assert!(is_transfer(&action(json!({"Transfer": {"deposit": "1"}}))));
        assert!(is_transfer(&action(json!({"FunctionCall": {"method_name": "ft_transfer", "deposit": "1"}}))));
        assert!(is_transfer(&action(json!({"FunctionCall": {"method_name": "ft_transfer_call", "deposit": "1"}}))));
        assert!(is_transfer(&action(json!({
            "Delegate": {
                "delegate_action": {
                    "sender_id": "sender.near",
                    "receiver_id": "usdt.tether-token.near",
                    "actions": [{"FunctionCall": {"method_name": "ft_transfer", "deposit": "1"}}]
                },
                "signature": "ed25519:signature"
            }
        }))));
        assert!(!is_transfer(&action(json!({"FunctionCall": {"method_name": "ft_deposit", "deposit": "0"}}))));
        assert!(!is_transfer(&action(json!({"FunctionCall": {"method_name": "execute_intents", "deposit": "0"}}))));
        assert!(!is_transfer(&action(json!("CreateAccount"))));
    }
}
