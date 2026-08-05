use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentLink {
    pub provider: PaymentProviderName,
    pub id: String,
}

impl PaymentLink {
    pub fn new(provider: PaymentProviderName, id: String) -> Self {
        Self { provider, id }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentMerchant {
    pub name: String,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Expired,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentOutcome {
    pub status: PaymentStatus,
    pub transaction_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentOptions {
    Quotes(PaymentQuotes),
    Outcome(PaymentOutcome),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentQuotes {
    pub merchant: PaymentMerchant,
    pub price: Option<PaymentPrice>,
    pub expires_at: Option<DateTime<Utc>>,
    pub quotes: Vec<PaymentQuote>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentPrice {
    pub symbol: String,
    pub value: String,
    pub decimals: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentQuote {
    pub id: String,
    pub payment_id: String,
    pub amount: PaymentAmount,
    pub expires_at: Option<DateTime<Utc>>,
    pub collect_data_url: Option<String>,
    pub provider_data: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum PaymentProviderName {
    SolanaPay,
    WalletConnectPay,
}

impl PaymentProviderName {
    pub fn has_status(&self) -> bool {
        match self {
            Self::WalletConnectPay => true,
            Self::SolanaPay => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentAmount {
    pub asset_id: AssetId,
    pub value: String,
    pub symbol: String,
    pub decimals: i32,
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
}
