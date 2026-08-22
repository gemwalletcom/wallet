use std::{collections::HashMap, error::Error};

use chrono::{DateTime, Utc};
use num_bigint::{BigUint, Sign};
use primitives::{AssetId, Chain, Transaction, TransactionState, TransactionType};

use crate::constants::{NATIVE_ASSET_ID, NEP_141_STANDARD};

use super::super::model::FastNearTransfer;

pub(in crate::rpc::indexer) fn map_address_transfer(
    transfer: FastNearTransfer,
    asset_id: AssetId,
    fees: &HashMap<String, BigUint>,
    address: &str,
) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let created_at = DateTime::<Utc>::from_timestamp_nanos(i64::try_from(transfer.block_timestamp)?);
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

pub(in crate::rpc::indexer) fn map_asset_id(asset_id: &str) -> Option<AssetId> {
    if asset_id == NATIVE_ASSET_ID {
        return Some(Chain::Near.as_asset_id());
    }

    asset_id
        .split_once(':')
        .filter(|(standard, token_id)| *standard == NEP_141_STANDARD && crate::address::is_valid_account_id(token_id))
        .map(|(_, token_id)| AssetId::from_token(Chain::Near, token_id))
}

#[cfg(test)]
mod tests {
    use primitives::asset_constants::NEAR_USDT_ASSET_ID;

    use super::*;

    #[test]
    fn test_map_asset_id() {
        assert_eq!(map_asset_id(NATIVE_ASSET_ID), Some(Chain::Near.as_asset_id()));
        assert_eq!(map_asset_id("nep141:usdt.tether-token.near"), Some(NEAR_USDT_ASSET_ID.clone()));

        for invalid_asset_id in [
            "native:near",
            "native:ethereum",
            "nep141:",
            "nep245:contract.near:nep141:token.near",
            "nep245:v2_1.omni.hot.tg:56_11111111111111111111",
        ] {
            assert_eq!(map_asset_id(invalid_asset_id), None);
        }
    }
}
