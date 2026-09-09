use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

use crate::asset_id::AssetId;

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
}
