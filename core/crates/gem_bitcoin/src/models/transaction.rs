use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_biguint_from_str, serialize_biguint};

use super::UInt64;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitcoinTransaction {
    pub block_height: UInt64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinTransactionBroadcastResult {
    pub error: Option<BitcoinTransactionBroadcastError>,
    pub result: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BitcoinTransactionBroadcastError {
    Plain(String),
    Detailed { message: String },
}

impl BitcoinTransactionBroadcastError {
    pub fn message(&self) -> &str {
        match self {
            Self::Plain(s) => s,
            Self::Detailed { message } => message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitcoinUTXO {
    pub txid: String,
    pub vout: i32,
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AddressDetails {
    pub transactions: Option<Vec<Transaction>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub txid: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub value_in: String,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub fees: BigUint,
    pub confirmations: Option<i64>,
    #[serde(rename = "confirmationETASeconds")]
    pub confirmation_eta_seconds: Option<i64>,
    pub block_time: i64,
    pub block_height: i64,
    pub vin: Vec<Input>,
    pub vout: Vec<Output>,
}

impl Transaction {
    pub fn is_confirmed(&self) -> bool {
        self.confirmations.map_or(self.block_height > 0, |confirmations| confirmations > 0)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>, // will be optional for Coinbase Input
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub n: i64,
    pub tx_id: Option<String>, // will be optional for Coinbase Input
    pub vout: Option<i64>,     // will be optional for Coinbase Input
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Output {
    pub is_address: bool,
    pub addresses: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_biguint_from_str")]
    pub value: BigUint,
    pub n: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_broadcast_error_string() {
        let json = r#"{"error": "-26: min relay fee not met, 432 < 576"}"#;
        let result: BitcoinTransactionBroadcastResult = serde_json::from_str(json).unwrap();

        assert!(result.result.is_none());
        assert_eq!(result.error.unwrap().message(), "-26: min relay fee not met, 432 < 576");
    }

    #[test]
    fn test_deserialize_confirmation_eta_seconds() {
        let json = r#"{
            "txid": "transaction-id",
            "value": "1",
            "valueIn": "2",
            "fees": "1",
            "confirmations": 0,
            "confirmationETASeconds": 698,
            "blockTime": 0,
            "blockHeight": -1,
            "vin": [],
            "vout": []
        }"#;

        let transaction: Transaction = serde_json::from_str(json).unwrap();

        assert_eq!(transaction.confirmation_eta_seconds, Some(698));
    }

    #[test]
    fn test_deserialize_broadcast_error_object() {
        let json = r#"{"error": {"message": "transaction already in block chain"}}"#;
        let result: BitcoinTransactionBroadcastResult = serde_json::from_str(json).unwrap();

        assert!(result.result.is_none());
        assert_eq!(result.error.unwrap().message(), "transaction already in block chain");
    }

    #[test]
    fn test_deserialize_broadcast_success() {
        let json = r#"{"result": "abc123def456"}"#;
        let result: BitcoinTransactionBroadcastResult = serde_json::from_str(json).unwrap();

        assert!(result.error.is_none());
        assert_eq!(result.result.unwrap(), "abc123def456");
    }
}
