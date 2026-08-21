use std::{collections::HashMap, error::Error};

use chrono::{DateTime, Utc};
use num_bigint::{BigUint, Sign};
use primitives::{AssetId, Chain, Transaction, TransactionState, TransactionType};

use crate::constants::{NATIVE_ASSET_ID, NEP_141_STANDARD};

use super::super::model::FastNearTransfer;

pub(in crate::rpc::indexer) fn map_address_transfer(
    transfer: FastNearTransfer,
    fees: &HashMap<String, BigUint>,
    address: &str,
) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let created_at = DateTime::<Utc>::from_timestamp_nanos(i64::try_from(transfer.block_timestamp)?);
    let asset_id = map_asset_id(&transfer.asset_id)?;
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
    let (from, to) = match (transfer.amount.sign(), transfer.other_account_id) {
        (Sign::Minus, Some(other_account_id)) => (transfer.account_id, other_account_id),
        (Sign::NoSign | Sign::Plus, Some(other_account_id)) => (other_account_id, transfer.account_id),
        (Sign::Minus, None) => (transfer.account_id, transfer.receipt_account_id),
        (Sign::NoSign | Sign::Plus, None) => (transfer.predecessor_id, transfer.account_id),
    };
    Ok(Transaction::new(
        hash,
        asset_id,
        from,
        to,
        None,
        TransactionType::Transfer,
        TransactionState::Confirmed,
        fee,
        Chain::Near.as_asset_id(),
        transfer.amount.magnitude().to_string(),
        None,
        None,
        created_at,
    ))
}

fn map_asset_id(asset_id: &str) -> Result<AssetId, Box<dyn Error + Send + Sync>> {
    if asset_id == NATIVE_ASSET_ID {
        return Ok(Chain::Near.as_asset_id());
    }
    let token_id = asset_id
        .split_once(':')
        .filter(|(standard, token_id)| *standard == NEP_141_STANDARD && crate::address::is_valid_account_id(token_id))
        .map(|(_, token_id)| token_id)
        .ok_or_else(|| format!("invalid FastNear asset id: {asset_id}"))?;
    Ok(AssetId::from_token(Chain::Near, token_id))
}
