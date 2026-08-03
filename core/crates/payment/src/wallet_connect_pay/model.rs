use primitives::{PaymentMerchant, PaymentStatus};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::wallet_connect_pay::error::WalletConnectPayError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAmount {
    pub unit: String,
    pub value: String,
    pub display: PaymentAmountDisplay,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentAmountDisplay {
    pub asset_symbol: String,
    pub asset_name: String,
    pub decimals: i32,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub network_name: Option<String>,
    #[serde(default)]
    pub network_icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInfo {
    pub status: PaymentStatus,
    pub amount: PaymentAmount,
    pub expires_at: i64,
    pub merchant: PaymentMerchant,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOption {
    pub id: String,
    pub account: String,
    pub amount: PaymentAmount,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub actions: Vec<WalletConnectPayAction>,
    #[serde(default)]
    pub collect_data: Option<PaymentCollectData>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentCollectData {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum WalletConnectPayAction {
    WalletRpc(WalletRpcAction),
    Build(BuildAction),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WalletRpcAction {
    pub chain_id: String,
    pub method: String,
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildAction {
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub enum WalletConnectPayActionResult {
    WalletRpc(Vec<Value>),
}

impl WalletConnectPayActionResult {
    pub fn wallet_rpc(result: String) -> Self {
        Self::WalletRpc(vec![Value::String(result)])
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOptionsRequest {
    pub accounts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentOptionsResponse {
    #[serde(default)]
    pub info: Option<PaymentInfo>,
    #[serde(default)]
    pub options: Option<Vec<PaymentOption>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchActionsRequest {
    pub option_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FetchActionsResponse {
    pub actions: Vec<WalletConnectPayAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentRequest {
    pub option_id: String,
    pub results: Vec<WalletConnectPayActionResult>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentStatusResponse {
    pub status: PaymentStatus,
    pub is_final: bool,
    #[serde(default)]
    pub poll_in_ms: Option<i64>,
    #[serde(default)]
    pub info: Option<PaymentResultInfo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentResultInfo {
    pub tx_id: String,
    #[serde(default)]
    pub option_amount: Option<PaymentAmount>,
}

impl TryFrom<WalletConnectPayAction> for WalletRpcAction {
    type Error = WalletConnectPayError;

    fn try_from(action: WalletConnectPayAction) -> Result<Self, Self::Error> {
        match action {
            WalletConnectPayAction::WalletRpc(wallet_rpc) => Ok(wallet_rpc),
            WalletConnectPayAction::Build(_) => Err(WalletConnectPayError::InvalidRequest("Payment action is not a wallet RPC call".to_string())),
        }
    }
}
