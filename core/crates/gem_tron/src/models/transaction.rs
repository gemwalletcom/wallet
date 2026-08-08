use crate::address::TronAddress;
use crate::address::serializer::deserialize as tron_address_deserialize;
use crate::address::serializer::optional as tron_address_optional;
use crate::models::TronContractType;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Transaction {
    #[serde(rename = "txID")]
    pub transaction_id: String,
    pub ret: Vec<ContractRet>,
    pub raw_data: TransactionData,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractRet {
    #[serde(rename = "contractRet")]
    pub contract_ret: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionData {
    pub contract: Vec<Contract>,
    pub fee_limit: Option<u64>,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Contract {
    #[serde(rename = "type")]
    #[serde(default, deserialize_with = "deserialize_contract_type_optional", skip_serializing_if = "Option::is_none")]
    pub contract_type: Option<TronContractType>,
    pub parameter: ContractParameter,
}

fn deserialize_contract_type_optional<'de, D>(deserializer: D) -> Result<Option<TronContractType>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.as_deref().and_then(|value| value.parse().ok()))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameter {
    pub type_url: String,
    pub value: ContractParameterValue,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContractParameterValue {
    pub amount: Option<u64>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub owner_address: Option<String>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub to_address: Option<String>,
    #[serde(default, deserialize_with = "tron_address_deserialize")]
    pub contract_address: Option<String>,
    pub data: Option<String>,
    pub frozen_balance: Option<u64>,
    pub unfreeze_balance: Option<u64>,
    pub resource: Option<String>,
    pub votes: Option<Vec<VoteInfo>>,
    pub support: Option<bool>,
    pub call_value: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VoteInfo {
    pub vote_address: String,
    pub vote_count: u64,
}

pub type BlockTransactionsInfo = Vec<TransactionReceiptData>;

pub const RECEIPT_OUT_OF_ENERGY: &str = "OUT_OF_ENERGY";
pub const RECEIPT_FAILED: &str = "FAILED";
pub const RECEIPT_REVERT: &str = "REVERT";

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceiptData {
    pub id: String,
    pub fee: Option<u64>,
    #[serde(rename = "blockNumber")]
    pub block_number: i64,
    #[serde(rename = "blockTimeStamp")]
    pub block_time_stamp: i64,
    pub result: Option<String>,
    pub receipt: TransactionReceipt,
    pub log: Option<Vec<TronLog>>,
    pub internal_transactions: Option<Vec<InternalTransaction>>,
}

impl TransactionReceiptData {
    pub fn is_failed(&self) -> bool {
        self.result.as_deref() == Some(RECEIPT_FAILED) || matches!(self.receipt.result.as_deref(), Some(RECEIPT_FAILED | RECEIPT_OUT_OF_ENERGY | RECEIPT_REVERT))
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransactionReceipt {
    pub result: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TronLog {
    #[serde(default, with = "tron_address_optional")]
    pub address: Option<TronAddress>,
    pub topics: Option<Vec<String>>,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InternalTransaction {
    #[serde(default, with = "tron_address_optional")]
    pub caller_address: Option<TronAddress>,
    #[serde(default, rename = "transferTo_address", with = "tron_address_optional")]
    pub transfer_to_address: Option<TronAddress>,
    #[serde(default, rename = "callValueInfo")]
    pub call_value_info: Vec<InternalTransactionCallValue>,
    #[serde(default)]
    pub rejected: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InternalTransactionCallValue {
    #[serde(default, rename = "callValue")]
    pub call_value: u64,
    /// TRC10 token id; `None` is native TRX.
    #[serde(default, rename = "tokenId", deserialize_with = "deserialize_token_id_optional")]
    pub token_id: Option<String>,
}

fn deserialize_token_id_optional<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    Ok(value.filter(|token_id| !token_id.is_empty()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronTransactionBroadcast {
    #[serde(rename = "txid")]
    pub txid: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_internal_transaction_call_value_token_id() {
        let native: InternalTransactionCallValue = serde_json::from_str(r#"{"callValue": 1, "tokenId": ""}"#).unwrap();
        assert_eq!(native.token_id, None);

        let trc10: InternalTransactionCallValue = serde_json::from_str(r#"{"callValue": 1, "tokenId": "1002000"}"#).unwrap();
        assert_eq!(trc10.token_id.as_deref(), Some("1002000"));
    }
}
