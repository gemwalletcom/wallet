use diesel::prelude::*;
use primitives::swap::SwapPartnerTransaction;

use crate::sql_types::{SwapProviderRow, SwapStatusRow};

#[derive(Debug, Insertable)]
#[diesel(table_name = crate::schema::swap_partner_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct NewSwapPartnerTransactionRow {
    pub provider: SwapProviderRow,
    pub provider_transaction_id: String,
    pub status: SwapStatusRow,
    pub from_asset_id: String,
    pub from_value: String,
    pub from_amount_usd: Option<f64>,
    pub to_asset_id: String,
    pub to_value: String,
    pub to_amount_usd: Option<f64>,
    pub referral_fee_asset_id: Option<String>,
    pub referral_fee_value: Option<String>,
    pub referral_fee_amount_usd: Option<f64>,
    pub from_transaction_hash: Option<String>,
    pub to_transaction_hash: Option<String>,
}

impl NewSwapPartnerTransactionRow {
    pub fn from_primitive(transaction: SwapPartnerTransaction) -> Self {
        let referral_fee = transaction.referral_fee;
        Self {
            provider: transaction.provider.into(),
            provider_transaction_id: transaction.provider_transaction_id,
            status: transaction.status.into(),
            from_asset_id: transaction.from_asset_id.to_string(),
            from_value: transaction.from_value,
            from_amount_usd: transaction.from_amount_usd,
            to_asset_id: transaction.to_asset_id.to_string(),
            to_value: transaction.to_value,
            to_amount_usd: transaction.to_amount_usd,
            referral_fee_asset_id: referral_fee.as_ref().map(|fee| fee.asset_id.to_string()),
            referral_fee_value: referral_fee.as_ref().map(|fee| fee.value.clone()),
            referral_fee_amount_usd: referral_fee.and_then(|fee| fee.amount_usd),
            from_transaction_hash: transaction.from_transaction_hash,
            to_transaction_hash: transaction.to_transaction_hash,
        }
    }
}
