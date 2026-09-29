use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::sql_types::{AssetId, SwapProviderRow, SwapStatusRow};

#[derive(Debug, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::transactions_swaps)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(treat_none_as_null = true)]
pub(crate) struct NewTransactionSwapRow {
    pub transaction_id: i64,
    pub provider: SwapProviderRow,
    pub status: SwapStatusRow,
    pub from_asset_id: AssetId,
    pub from_amount: f64,
    pub from_amount_usd: Option<f64>,
    pub to_asset_id: AssetId,
    pub to_amount: f64,
    pub to_amount_usd: Option<f64>,
    pub referral_fee_asset_id: AssetId,
    pub referral_fee_amount_usd: Option<f64>,
    pub created_at: NaiveDateTime,
}
