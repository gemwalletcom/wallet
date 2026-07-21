use alloy_primitives::hex;
use gem_jsonrpc::types::ToJsonRpcRequest;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Value, json};

use crate::method;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionObject {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "gasPrice")]
    pub gas_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxFeePerGas")]
    pub max_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "maxPriorityFeePerGas")]
    pub max_priority_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(serialize_with = "serialize_calldata")]
    pub data: String,
}

fn serialize_calldata<S: Serializer>(value: &str, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(if value.is_empty() { "0x" } else { value })
}

impl TransactionObject {
    pub fn new_call(to: &str, data: Vec<u8>) -> Self {
        Self {
            from: None,
            to: to.to_string(),
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            value: None,
            data: hex::encode_prefixed(data),
        }
    }

    pub fn new_call_to_value(to: &str, value: &str, data: Vec<u8>) -> Self {
        Self {
            value: Some(value.to_string()),
            ..Self::new_call(to, data)
        }
    }

    pub fn new_call_with_from(from: &str, to: &str, data: Vec<u8>) -> Self {
        Self {
            from: Some(from.to_string()),
            ..Self::new_call(to, data)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockParameter {
    // hexadecimal block number
    Number(&'static str),
    Latest,
    Earliest,
    Pending,
    Safe,
    Finalized,
}

impl From<&BlockParameter> for &'static str {
    fn from(val: &BlockParameter) -> Self {
        match val {
            BlockParameter::Number(val) => val,
            BlockParameter::Latest => "latest",
            BlockParameter::Earliest => "earliest",
            BlockParameter::Pending => "pending",
            BlockParameter::Safe => "safe",
            BlockParameter::Finalized => "finalized",
        }
    }
}

impl From<&BlockParameter> for serde_json::Value {
    fn from(val: &BlockParameter) -> Self {
        let str: &str = val.into();
        serde_json::Value::String(str.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum EthereumRpc {
    BlockNumber,
    Call(TransactionObject, BlockParameter),
    ChainId,
    EstimateGas(Value, BlockParameter),
    FeeHistory { blocks: u64, reward_percentiles: Vec<u64> },
    GasPrice,
    GetBalance(String, BlockParameter),
    GetBlockByNumber(u64),
    GetBlockReceipts(u64),
    GetCode(String, BlockParameter),
    GetTransactionByHash(String),
    GetTransactionCount(String, BlockParameter),
    GetTransactionReceipt(String),
    SendRawTransaction(String),
    Syncing,
    TraceCall(TransactionObject, BlockParameter),
}

impl ToJsonRpcRequest for EthereumRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::BlockNumber => method::ETH_BLOCK_NUMBER,
            Self::Call(_, _) => method::ETH_CALL,
            Self::ChainId => method::ETH_CHAIN_ID,
            Self::EstimateGas(_, _) => method::ETH_ESTIMATE_GAS,
            Self::FeeHistory { .. } => method::ETH_FEE_HISTORY,
            Self::GasPrice => method::ETH_GAS_PRICE,
            Self::GetBalance(_, _) => method::ETH_GET_BALANCE,
            Self::GetBlockByNumber(_) => method::ETH_GET_BLOCK_BY_NUMBER,
            Self::GetBlockReceipts(_) => method::ETH_GET_BLOCK_RECEIPTS,
            Self::GetCode(_, _) => method::ETH_GET_CODE,
            Self::GetTransactionByHash(_) => method::ETH_GET_TRANSACTION_BY_HASH,
            Self::GetTransactionCount(_, _) => method::ETH_GET_TRANSACTION_COUNT,
            Self::GetTransactionReceipt(_) => method::ETH_GET_TRANSACTION_RECEIPT,
            Self::SendRawTransaction(_) => method::ETH_SEND_RAW_TRANSACTION,
            Self::Syncing => method::ETH_SYNCING,
            Self::TraceCall(_, _) => method::TRACE_CALL,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::BlockNumber | Self::ChainId | Self::GasPrice | Self::Syncing => json!([]),
            Self::Call(transaction, block) => json!([transaction, Value::from(block)]),
            Self::EstimateGas(transaction, block) => json!([transaction, Value::from(block)]),
            Self::FeeHistory { blocks, reward_percentiles } => {
                json!([format!("0x{blocks:x}"), Value::from(&BlockParameter::Latest), reward_percentiles])
            }
            Self::GetBalance(address, block) | Self::GetCode(address, block) | Self::GetTransactionCount(address, block) => {
                json!([address, Value::from(block)])
            }
            Self::GetBlockByNumber(block_number) => json!([format!("0x{block_number:x}"), true]),
            Self::GetBlockReceipts(block_number) => json!([format!("0x{block_number:x}")]),
            Self::GetTransactionByHash(hash) | Self::GetTransactionReceipt(hash) | Self::SendRawTransaction(hash) => json!([hash]),
            Self::TraceCall(transaction, block) => json!([transaction, ["trace", "stateDiff"], Value::from(block)]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_request(rpc: EthereumRpc, method: &str, params: Value) {
        let request = rpc.to_jsonrpc_request(42);
        assert_eq!(request.id, 42);
        assert_eq!(request.method, method);
        assert_eq!(request.params, params);
    }

    #[test]
    fn test_encode_call() {
        let request = TransactionObject::new_call_with_from("0x46340b20830761efd32832a74d7169b29feb9758", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", vec![]);
        let encoded = serde_json::to_string(&request).unwrap();

        assert_eq!(
            encoded,
            r#"{"from":"0x46340b20830761efd32832a74d7169b29feb9758","to":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","data":"0x"}"#
        );
    }

    #[test]
    fn test_serialize_empty_calldata_as_0x() {
        let transaction = TransactionObject {
            data: String::new(),
            ..TransactionObject::new_call("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48", vec![])
        };

        assert_eq!(
            serde_json::to_string(&transaction).unwrap(),
            r#"{"to":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","data":"0x"}"#
        );
    }

    #[test]
    fn builds_call_request() {
        assert_request(
            EthereumRpc::Call(TransactionObject::new_call("0x1234", vec![0xab, 0xcd]), BlockParameter::Latest),
            method::ETH_CALL,
            json!([{"to": "0x1234", "data": "0xabcd"}, "latest"]),
        );
    }

    #[test]
    fn encodes_block_quantities_as_hex() {
        assert_request(EthereumRpc::GetBlockByNumber(26), method::ETH_GET_BLOCK_BY_NUMBER, json!(["0x1a", true]));
    }

    #[test]
    fn builds_fee_history_request() {
        assert_request(
            EthereumRpc::FeeHistory {
                blocks: 10,
                reward_percentiles: vec![25, 75],
            },
            method::ETH_FEE_HISTORY,
            json!(["0xa", "latest", [25, 75]]),
        );
    }

    #[test]
    fn builds_broadcast_request() {
        assert_request(EthereumRpc::SendRawTransaction("0xsigned".into()), method::ETH_SEND_RAW_TRANSACTION, json!(["0xsigned"]));
    }
}
