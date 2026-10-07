use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::sql_types::{AssetId, PerpetualDirectionRow, PerpetualProviderRow, TransactionType};

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions_perpetuals)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewTransactionPerpetualRow {
    pub transaction_id: i64,
    pub provider: PerpetualProviderRow,
    pub asset_id: AssetId,
    pub kind: TransactionType,
    pub direction: PerpetualDirectionRow,
    pub size_usd: f64,
    pub pnl_usd: f64,
    pub referral_fee_amount_usd: f64,
    pub created_at: NaiveDateTime,
}
