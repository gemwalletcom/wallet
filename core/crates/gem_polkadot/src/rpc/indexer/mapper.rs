use std::error::Error;

use primitives::{Chain, DateTime, Transaction, TransactionState, TransactionType};

use super::SubscanTransfer;

pub(super) fn map_transaction(transfer: SubscanTransfer) -> Result<Transaction, Box<dyn Error + Send + Sync>> {
    let state = if transfer.success { TransactionState::Confirmed } else { TransactionState::Failed };
    let created_at = DateTime::from_timestamp(transfer.block_timestamp);
    Ok(Transaction::new(
        transfer.hash,
        Chain::Polkadot.as_asset_id(),
        transfer.from,
        transfer.to,
        None,
        TransactionType::Transfer,
        state,
        transfer.fee,
        Chain::Polkadot.as_asset_id(),
        transfer.amount,
        None,
        None,
        created_at,
    ))
}
