use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct AcrossDepositsQuery {
    pub recipient: String,
    pub limit: usize,
    pub skip: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AcrossDeposit {
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub depositor: String,
    pub input_token: String,
    pub input_amount: String,
    pub output_token: String,
    pub message: String,
    pub status: String,
    pub deposit_tx_hash: String,
    pub fill_tx: Option<String>,
    pub deposit_block_timestamp: DateTime<Utc>,
    pub input_price_usd: Option<String>,
    pub output_price_usd: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AcrossPartnerCursor {
    pub handler: usize,
    pub skip: usize,
    pub walk_started_at: Option<i64>,
    pub since: Option<i64>,
}
