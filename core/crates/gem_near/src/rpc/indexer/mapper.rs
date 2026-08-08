use std::{collections::HashMap, error::Error};

use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use primitives::{Chain, Transaction, TransactionState, TransactionType};

use super::FastNearTransfer;

pub(super) fn map_transaction(transfer: FastNearTransfer, fees: &HashMap<String, BigUint>, address: &str) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let timestamp = i64::try_from(transfer.block_timestamp)?;
    let created_at = DateTime::<Utc>::from_timestamp_nanos(timestamp);
    let value = transfer.amount.magnitude().to_string();
    let fee = if transfer.signer_id == address && transfer.predecessor_id == address {
        let transaction_id = transfer.transaction_id.as_ref().ok_or("missing FastNear sender transaction id")?;
        fees.get(transaction_id).ok_or("missing FastNear sender transaction details")?.to_string()
    } else {
        "0".to_string()
    };
    let hash = if transfer.predecessor_id == transfer.signer_id {
        transfer.transaction_id.ok_or("missing FastNear transaction id")?
    } else {
        transfer.receipt_id
    };
    Ok(Transaction::new(
        hash,
        Chain::Near.as_asset_id(),
        transfer.predecessor_id,
        transfer.receipt_account_id,
        None,
        TransactionType::Transfer,
        TransactionState::Confirmed,
        fee,
        Chain::Near.as_asset_id(),
        value,
        None,
        None,
        created_at,
    ))
}
