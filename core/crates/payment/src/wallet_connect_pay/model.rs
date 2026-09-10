use num_bigint::BigUint;
use primitives::{AssetId, ChainAddress, PaymentStatus, PaymentPrice};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::PaymentError;

#[derive(Debug, Clone, PartialEq)]
pub(super) struct PaymentAction {
    pub account: ChainAddress,
    pub recipient: String,
    pub value: BigUint,
    pub data: String,
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

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Invoice {
    pub merchant: Merchant,
    pub price: PaymentPrice,
    pub quotes: Vec<Quote>,
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
pub(super) struct PaymentAmount {
    pub unit: String,
    pub value: String,
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
    pub value: String,
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
    pub amount: PaymentAmount,
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
    pub(super) fn wallet_rpc(result: String) -> Self {
        Self::WalletRpc(vec![Value::String(result)])
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
    #[serde(default)]
    pub info: Option<PaymentInfo>,
    #[serde(default)]
    pub options: Option<Vec<PaymentOption>>,
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
}

impl TryFrom<WalletConnectPayAction> for WalletRpcAction {
    type Error = PaymentError;

    fn try_from(action: WalletConnectPayAction) -> Result<Self, Self::Error> {
        match action {
            WalletConnectPayAction::WalletRpc(wallet_rpc) => Ok(wallet_rpc),
            WalletConnectPayAction::Build(_) => Err(PaymentError::InvalidRequest {
                reason: "Payment action is not a wallet RPC call".to_string(),
            }),
        }
    }
}
