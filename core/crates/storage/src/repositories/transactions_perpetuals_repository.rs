use chrono::NaiveDateTime;
use diesel::prelude::*;
use primitives::{AssetId, PerpetualDirection, PerpetualProvider, TransactionId, TransactionType};

use crate::models::NewTransactionPerpetualRow;
use crate::repositories::transactions_repository::transaction_row_id;
use crate::{DatabaseClient, DatabaseError};

#[derive(Debug, Clone, PartialEq)]
pub struct TransactionPerpetualRecord {
    pub provider: PerpetualProvider,
    pub asset_id: AssetId,
    pub kind: TransactionType,
    pub direction: PerpetualDirection,
    pub size_usd: f64,
    pub referral_fee_amount_usd: f64,
    pub created_at: NaiveDateTime,
}

pub trait TransactionsPerpetualsRepository {
    fn upsert_transaction_perpetual(&mut self, id: &TransactionId, record: TransactionPerpetualRecord) -> Result<usize, DatabaseError>;
}

impl TransactionsPerpetualsRepository for DatabaseClient {
    fn upsert_transaction_perpetual(&mut self, id: &TransactionId, record: TransactionPerpetualRecord) -> Result<usize, DatabaseError> {
        use crate::schema::transactions_perpetuals::dsl::*;
        let row = NewTransactionPerpetualRow {
            transaction_id: transaction_row_id(self, id)?,
            provider: record.provider.into(),
            asset_id: record.asset_id.into(),
            kind: record.kind.into(),
            direction: record.direction.into(),
            size_usd: record.size_usd,
            referral_fee_amount_usd: record.referral_fee_amount_usd,
            created_at: record.created_at,
        };
        Ok(diesel::insert_into(transactions_perpetuals).values(&row).on_conflict(transaction_id).do_update().set(&row).execute(&mut self.connection)?)
    }
}
