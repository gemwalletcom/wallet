use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::asset_id::AssetId;
use crate::payment_decoder::WALLET_CONNECT_PAY_HOST;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum Payment {
    Request { request: PaymentRequest },
    Link { link: PaymentLink },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentAmount {
    ExactValue {
        value: String,
    },
    AtomicValue {
        #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
        value: BigUint,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentRequest {
    pub address: String,
    pub amount: Option<PaymentAmount>,
    pub memo: Option<String>,
    pub label: Option<String>,
    pub references: Option<Vec<String>>,
    pub asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "content", rename_all = "camelCase")]
pub enum PaymentLink {
    SolanaPay { url: String },
    WalletConnectPay { payment_id: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInvoice {
    pub link: PaymentLink,
    pub merchant: PaymentMerchant,
    pub price: Option<PaymentPrice>,
    pub quotes: Vec<PaymentQuote>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMerchant {
    pub name: String,
    pub icon: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentPrice {
    pub currency: String,
    pub amount: f64,
}

impl PaymentLink {
    pub fn url(&self) -> String {
        match self {
            Self::SolanaPay { url } => url.clone(),
            Self::WalletConnectPay { payment_id } => format!("https://{WALLET_CONNECT_PAY_HOST}/?pid={payment_id}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentQuote {
    pub id: String,
    pub asset_id: AssetId,
    #[serde(serialize_with = "serde_serializers::serialize_biguint", deserialize_with = "serde_serializers::deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Expired,
    Cancelled,
}

