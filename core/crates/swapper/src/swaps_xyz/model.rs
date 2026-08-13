use primitives::contract_constants::EVM_ZERO_ADDRESS;
use serde::{Deserialize, Serialize};

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
    pub value: String,
    pub chain_id: u64,
    pub chain_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    pub amount: String,
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
