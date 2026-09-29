use diesel::prelude::*;
use primitives::{AssetId, SwapProvider, TransactionId, swap::SwapStatus};

use crate::models::NewTransactionSwapRow;
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionSwapRecord {
    pub provider: SwapProvider,
    pub status: SwapStatus,
    pub from_asset_id: AssetId,
    pub from_amount_usd: Option<f64>,
    pub to_asset_id: AssetId,
    pub to_amount_usd: Option<f64>,
    pub referral_fee_asset_id: AssetId,
    pub referral_fee_amount_usd: Option<f64>,
}

pub trait TransactionsSwapsRepository {
    fn upsert_transaction_swap(&mut self, id: &TransactionId, record: TransactionSwapRecord) -> Result<usize, DatabaseError>;
}

impl TransactionsSwapsRepository for DatabaseClient {
    fn upsert_transaction_swap(&mut self, id: &TransactionId, record: TransactionSwapRecord) -> Result<usize, DatabaseError> {
        use crate::schema::transactions_swaps::dsl::*;
        let row = NewTransactionSwapRow {
            transaction_id: transaction_row_id(self, id)?,
            provider: record.provider.into(),
            status: record.status.into(),
            from_asset_id: record.from_asset_id.into(),
            from_amount_usd: record.from_amount_usd,
            to_asset_id: record.to_asset_id.into(),
            to_amount_usd: record.to_amount_usd,
            referral_fee_asset_id: record.referral_fee_asset_id.into(),
            referral_fee_amount_usd: record.referral_fee_amount_usd,
        };
        Ok(diesel::insert_into(transactions_swaps).values(&row).on_conflict(transaction_id).do_update().set(&row).execute(&mut self.connection)?)
    }
}

fn transaction_row_id(client: &mut DatabaseClient, transaction_id: &TransactionId) -> Result<i64, DatabaseError> {
    use crate::schema::transactions::dsl::*;
    Ok(transactions.filter(chain.eq(transaction_id.chain.as_ref())).filter(hash.eq(&transaction_id.hash)).select(id).first(&mut client.connection)?)
}
