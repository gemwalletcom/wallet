use std::error::Error;

use chrono::{DateTime, Utc};
use number_formatter::BigNumberFormatter;
use primitives::{Chain, Transaction, TransactionState, TransactionType};

use super::SubscanTransfer;

const POLKADOT_DECIMALS: u32 = 10;

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
        BigNumberFormatter::value_from_amount_biguint(&transfer.fee, POLKADOT_DECIMALS)?.to_string(),
        Chain::Polkadot.as_asset_id(),
        BigNumberFormatter::value_from_amount_biguint(&transfer.amount, POLKADOT_DECIMALS)?.to_string(),
        None,
        None,
        created_at,
    ))
}
