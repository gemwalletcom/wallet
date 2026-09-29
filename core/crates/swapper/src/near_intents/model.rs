use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::deserialize_biguint_from_str;

pub const DEPOSIT_TYPE_ORIGIN: &str = "ORIGIN_CHAIN";
pub const RECIPIENT_TYPE_DESTINATION: &str = "DESTINATION_CHAIN";
pub const DEFAULT_WAIT_TIME_MS: u32 = 1_024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppFee {
    pub recipient: String,
    pub fee: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    pub origin_asset: String,
    pub destination_asset: String,
    pub amount: String,
    pub referral: String,
    pub recipient: String,
    pub swap_type: SwapType,
    pub slippage_tolerance: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_fees: Option<Vec<AppFee>>,
    pub deposit_type: String,
    pub refund_to: String,
    pub refund_type: String,
    pub recipient_type: String,
    pub deadline: String,
    pub quote_waiting_time_ms: Option<u32>,
    pub dry: bool,
    pub deposit_mode: DepositMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SwapType {
    ExactInput,
    FlexInput,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DepositMode {
    #[default]
    Simple,
    Memo,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    pub quote_request: QuoteRequest,
    pub quote: Quote,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuoteResponseError {
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum QuoteResponseResult {
    Ok(Box<QuoteResponse>),
    Err(QuoteResponseError),
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub deposit_address: Option<String>,
    pub deposit_memo: Option<String>,
    pub deposit_mode: Option<DepositMode>,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount_in: BigUint,
    pub amount_in_formatted: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub min_amount_in: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub amount_out: BigUint,
    pub amount_out_formatted: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub min_amount_out: BigUint,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub withdraw_fee: BigUint,
    pub deadline: Option<String>,
    pub time_when_inactive: Option<String>,
    pub time_estimate: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerTransaction {
    pub deposit_address: String,
    pub status: String,
    pub origin_asset: String,
    pub destination_asset: String,
    pub amount_in: String,
    pub amount_out: String,
    pub origin_chain_tx_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerTransactionsQuery {
    pub search: String,
    pub number_of_transactions: usize,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerPartnerTransaction {
    pub deposit_address: String,
    pub deposit_memo: Option<String>,
    pub status: String,
    pub origin_asset: String,
    pub destination_asset: String,
    pub amount_in: String,
    pub amount_in_usd: String,
    pub amount_out: String,
    pub amount_out_usd: String,
    pub senders: Vec<String>,
    pub refund_to: String,
    pub recipient: String,
    pub app_fees: Vec<ExplorerAppFee>,
    pub origin_chain_tx_hashes: Vec<String>,
    pub destination_chain_tx_hashes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExplorerAppFee {
    pub recipient: String,
    pub fee: u32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerPartnerTransactionsQuery {
    pub referral: String,
    pub number_of_transactions: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp_unix: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deposit_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deposit_memo: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerPartnerCursor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk_started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deposit_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_deposit_memo: Option<String>,
}
