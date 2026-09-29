use num_bigint::BigUint;
use primitives::contract_constants::EVM_ZERO_ADDRESS;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

use super::chain::SwapsXyzChain;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionRequest {
    pub action_type: String,
    pub sender: String,
    pub src_chain_id: u64,
    pub src_token: String,
    pub dst_chain_id: u64,
    pub dst_token: String,
    pub slippage: u32,
    pub amount: String,
    pub swap_direction: String,
    pub recipient: String,
    pub refund_to: String,
    pub return_deposit_address: bool,
    pub app_fees: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppFee {
    pub bps: u32,
    pub receiver_address: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionResponse {
    pub vm_id: String,
    pub tx: AltVmTransaction,
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
    pub application_fee: TokenAmount,
    pub estimated_tx_time: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AltVmTransaction {
    pub to: String,
    pub to_extra: Option<String>,
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub value: BigUint,
    pub chain_id: u64,
    pub chain_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    #[serde(deserialize_with = "deserialize_biguint_from_str", serialize_with = "serialize_biguint")]
    pub amount: BigUint,
    pub chain_id: u64,
    pub address: String,
    pub decimals: u32,
    pub is_native: bool,
}

impl TokenAmount {
    pub(super) fn native_chain(&self) -> Option<SwapsXyzChain> {
        let chain = SwapsXyzChain::from_id(self.chain_id)?;
        (self.is_native && self.address == EVM_ZERO_ADDRESS && self.decimals == chain.decimals()).then_some(chain)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsResponse {
    pub paths: Vec<Path>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Path {
    pub chain_id: u64,
    pub tokens: Vec<PathToken>,
    pub supports_exact_amount_in: bool,
    pub amount_limits: AmountLimits,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathToken {
    pub address: String,
    pub is_native: bool,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmountLimits {
    pub min_amount: String,
    pub max_amount: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    pub status: String,
    pub action_response: Option<StatusActionResponse>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusActionResponse {
    pub amount_in: TokenAmount,
    pub amount_out: TokenAmount,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsQuery {
    pub src_chain_id: u64,
    pub src_token: String,
    pub dst_chain_id: u64,
}

impl PathsQuery {
    pub fn native(source_chain_id: u64, destination_chain_id: u64) -> Self {
        Self {
            src_chain_id: source_chain_id,
            src_token: EVM_ZERO_ADDRESS.to_string(),
            dst_chain_id: destination_chain_id,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusQuery {
    pub tx_hash: String,
    pub chain_id: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionsResponse {
    pub txs: Vec<PartnerTransaction>,
    pub cursor: TransactionsCursor,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TransactionsCursor {
    pub next: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerTransaction {
    pub tx_id: String,
    pub status: String,
    pub sender: String,
    pub src_tx: Option<OnchainTransaction>,
    pub dst_tx: Option<OnchainTransaction>,
    pub action_request: Option<PartnerActionRequest>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerActionRequest {
    #[serde(default)]
    pub app_fees: Vec<AppFee>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnchainTransaction {
    pub payment_token: PaymentToken,
    pub to_address: Option<String>,
    pub tx_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentToken {
    pub chain_id: u64,
    pub is_native: bool,
    pub token_address: Option<String>,
    pub amount: String,
    pub usd_amount: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionsQuery {
    pub limit: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartnerCursor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub walk_started_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}
