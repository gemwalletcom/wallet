use num_bigint::BigUint;
use primitives::swap::ApprovalData;
use primitives::{AssetId, ChainAddress, PaymentPrice, PaymentStatus, WalletConnectionMethods};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_serializers::deserialize_biguint_from_str;

use crate::error::PaymentError;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PaymentAction {
    Send(PaymentSend),
    Sign(PaymentSign),
    ApproveAndSign { approval: ApprovalData, sign: PaymentSign },
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PaymentSend {
    pub recipient: String,
    pub data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PaymentSign {
    pub recipient: String,
    pub typed_data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct TypedDataTransfer {
    pub token: String,
    pub amount: BigUint,
    pub from: Option<String>,
    pub recipient: String,
    pub typed_data: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Quote {
    pub id: String,
    pub account: ChainAddress,
    pub asset_id: AssetId,
    pub value: BigUint,
    pub collect_data_url: Option<String>,
    pub actions: Vec<WalletConnectPayAction>,
}

impl Quote {
    pub fn token(&self) -> &str {
        self.asset_id.token_id.as_deref().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Invoice {
    pub merchant: Merchant,
    pub price: PaymentPrice,
    pub quotes: Vec<Quote>,
    pub collect_data_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Options {
    Invoice(Invoice),
    Status { status: PaymentStatus },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Merchant {
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentOptionAmount {
    pub unit: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentInfo {
    pub status: PaymentStatus,
    pub merchant: Merchant,
    pub amount: PaymentPriceAmount,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentPriceAmount {
    pub unit: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub display: PaymentPriceDisplay,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentPriceDisplay {
    pub decimals: u32,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentOption {
    pub id: String,
    pub account: String,
    pub amount: PaymentOptionAmount,
    #[serde(default)]
    pub actions: Vec<WalletConnectPayAction>,
    #[serde(default)]
    pub collect_data: Option<PaymentCollectData>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentCollectData {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub(super) enum WalletConnectPayAction {
    WalletRpc(WalletRpcAction),
    Build(BuildAction),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PaymentActions {
    Ready(Vec<WalletConnectPayAction>),
    CollectData,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(super) struct WalletRpcAction {
    pub chain_id: String,
    pub method: String,
    pub params: Value,
}

impl WalletRpcAction {
    pub fn method(&self) -> Option<WalletConnectionMethods> {
        serde_json::from_value(Value::String(self.method.clone())).ok()
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub(super) struct BuildAction {
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", content = "data", rename_all = "camelCase")]
pub(super) enum WalletConnectPayActionResult {
    WalletRpc(Vec<Value>),
}

impl WalletConnectPayActionResult {
    pub(super) fn wallet_rpc(action_result: String) -> Self {
        Self::WalletRpc(vec![Value::String(action_result)])
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentOptionsRequest<'a> {
    pub accounts: &'a [String],
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentOptionsResponse {
    pub info: PaymentInfo,
    #[serde(default)]
    pub options: Option<Vec<PaymentOption>>,
    #[serde(default)]
    pub collect_data: Option<PaymentCollectData>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FetchActionsRequest {
    pub option_id: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct FetchActionsResponse {
    pub actions: Vec<WalletConnectPayAction>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConfirmPaymentRequest {
    pub option_id: String,
    pub results: Vec<WalletConnectPayActionResult>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentStatusResponse {
    pub status: PaymentStatus,
    #[serde(default)]
    pub info: Option<PaymentStatusInfo>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PaymentStatusInfo {
    #[serde(default)]
    pub tx_id: Option<String>,
}

impl TryFrom<WalletConnectPayAction> for WalletRpcAction {
    type Error = PaymentError;

    fn try_from(action: WalletConnectPayAction) -> Result<Self, Self::Error> {
        match action {
            WalletConnectPayAction::WalletRpc(wallet_rpc) => Ok(wallet_rpc),
            WalletConnectPayAction::Build(_) => Err(PaymentError::invalid_request("Payment action is not a wallet RPC call")),
        }
    }
}
