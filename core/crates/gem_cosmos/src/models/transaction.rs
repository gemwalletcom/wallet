use gem_encoding::decode_base64;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use std::str;

use super::message::{AuthInfo, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastRequest {
    pub mode: String,
    pub tx_bytes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BroadcastResponse {
    pub tx_response: Option<TransactionResult>,
    pub code: Option<i32>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub txhash: String,
    pub code: i32,
    pub raw_log: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub tx: TransactionResponseTx,
    pub tx_response: TransactionResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionsResponse {
    pub txs: Vec<TransactionResponseTx>,
    pub tx_responses: Vec<TransactionResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseTx {
    pub body: TransactionBody,
    pub auth_info: Option<AuthInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBody {
    pub memo: String,
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponseData {
    pub code: i64,
    pub txhash: String,
    pub events: Vec<TransactionEvent>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub attributes: Vec<TransactionEventAttribute>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionEventAttribute {
    pub key: String,
    pub value: Option<String>,
}

impl TransactionResponse {
    pub fn get_rewards_value(&self, denom: &str) -> Option<BigUint> {
        //base64 decoding added for sei/celestia. This is a temporary solution until the issue is resolved in the cosmos-sdk
        self.tx_response
            .events
            .iter()
            .filter(|event| event.event_type == crate::constants::EVENTS_WITHDRAW_REWARDS_TYPE)
            .flat_map(|event| &event.attributes)
            .filter(|attribute| {
                decode_base64(&attribute.key)
                    .ok()
                    .and_then(|value| str::from_utf8(&value).ok().map(|value| value == crate::constants::EVENTS_ATTRIBUTE_AMOUNT))
                    .unwrap_or(attribute.key == crate::constants::EVENTS_ATTRIBUTE_AMOUNT)
            })
            .try_fold(BigUint::ZERO, |total, attribute| {
                let value = attribute.value.as_deref().unwrap_or_default();
                let decoded = decode_base64(value).ok();
                let value = decoded.as_deref().and_then(|value| str::from_utf8(value).ok()).unwrap_or(value);
                let amount = value.split(',').find_map(|value| value.strip_suffix(denom)).unwrap_or("0").parse::<BigUint>().ok()?;
                Some(total + amount)
            })
    }
}
