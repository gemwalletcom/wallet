use std::error::Error;

use chrono::{DateTime, Utc};
use primitives::{Chain, Transaction, TransactionState, TransactionType};

use super::SubscanTransfer;

pub(super) fn map_transaction(transfer: SubscanTransfer) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let state = if transfer.success { TransactionState::Confirmed } else { TransactionState::Failed };
    let created_at = DateTime::<Utc>::from_timestamp(transfer.block_timestamp, 0).ok_or("invalid Subscan block timestamp")?;
    Ok(Transaction::new(
        transfer.hash,
        Chain::Polkadot.as_asset_id(),
        transfer.from,
        transfer.to,
        None,
        TransactionType::Transfer,
        state,
        transfer.fee.to_string(),
        Chain::Polkadot.as_asset_id(),
        transfer.amount.to_string(),
        None,
        None,
        created_at,
    ))
}
