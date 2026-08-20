use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use typeshare::typeshare;

use crate::asset_id::AssetId;
use crate::chain::Chain;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum Payment {
    Request(PaymentRequest),
    Link(PaymentLink),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentAmount {
    ExactValue(String),
    AtomicValue(#[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")] BigUint),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequest {
    pub address: String,
    pub amount: Option<PaymentAmount>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentLink {
    SolanaPay(String),
    WalletConnectPay(String),
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
pub struct PaymentQuote {
    pub id: String,
    pub link: PaymentLink,
    pub asset_id: AssetId,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
    pub expires_at: Option<DateTime<Utc>>,
    pub collect_data_url: Option<String>,
    pub provider_data: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentPrice {
    pub symbol: String,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
    pub decimals: i32,
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
pub enum PaymentAction {
    Send {
        chain: Chain,
        recipient: String,
        #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
        value: BigUint,
        data: String,
    },
}

impl PaymentAction {
    pub fn chain(&self) -> Chain {
        match self {
            Self::Send { chain, .. } => *chain,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct PaymentQuoteData {
    pub quote: PaymentQuote,
    pub action: PaymentAction,
}
