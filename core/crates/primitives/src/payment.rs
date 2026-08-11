use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::asset_id::AssetId;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum Payment {
    Request(PaymentRequest),
    Link(PaymentLink),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequest {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
}

impl PaymentRequest {
    pub fn new_address(address: &str) -> Self {
        Self {
            address: address.to_string(),
            amount: None,
            memo: None,
            asset_id: None,
        }
    }

    pub fn with_asset(self, asset_id: AssetId) -> Self {
        Self { asset_id: Some(asset_id), ..self }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentLink {
    SolanaPay(String),
    WalletConnectPay(String),
}
