use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::json;

use crate::method;

#[derive(Clone, Debug)]
pub enum NearRpc {
    CallFunction { contract_id: String, method_name: String, args_base64: String },
    GetAccount { account_id: String },
    GetAccountAccessKey { address: String, public_key: String },
    GetGasPrice,
    GetLatestBlock,
    GetProtocolConfig,
    GetStatus,
    GetTransactionStatus { transaction_hash: String, sender_account_id: String },
    SendTransaction { signed_transaction: String },
}

impl ToJsonRpcRequest for NearRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::CallFunction { .. } | Self::GetAccount { .. } | Self::GetAccountAccessKey { .. } => method::QUERY,
            Self::GetGasPrice => method::GAS_PRICE,
            Self::GetLatestBlock => method::BLOCK,
            Self::GetProtocolConfig => method::PROTOCOL_CONFIG,
            Self::GetStatus => method::STATUS,
            Self::GetTransactionStatus { .. } => method::TRANSACTION,
            Self::SendTransaction { .. } => method::SEND_TRANSACTION,
        }
    }

    fn params(&self) -> serde_json::Value {
        match self {
            Self::CallFunction {
                contract_id,
                method_name,
                args_base64,
            } => json!({
                "request_type": "call_function",
                "finality": "final",
                "account_id": contract_id,
                "method_name": method_name,
                "args_base64": args_base64
            }),
            Self::GetAccount { account_id } => json!({
                "request_type": "view_account",
                "finality": "final",
                "account_id": account_id
            }),
            Self::GetAccountAccessKey { address, public_key } => json!({
                "request_type": "view_access_key",
                "finality": "final",
                "account_id": address,
                "public_key": public_key
            }),
            Self::GetGasPrice => json!([null]),
            Self::GetLatestBlock => json!({"finality": "final"}),
            Self::GetProtocolConfig => json!({"finality": "final"}),
            Self::GetStatus => json!([]),
            Self::GetTransactionStatus {
                transaction_hash,
                sender_account_id,
            } => json!({
                "tx_hash": transaction_hash,
                "sender_account_id": sender_account_id,
                "wait_until": "EXECUTED"
            }),
            Self::SendTransaction { signed_transaction } => json!({"signed_tx_base64": signed_transaction}),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_request(rpc: NearRpc, method: &str, params: serde_json::Value) {
        let request = rpc.to_jsonrpc_request(42);
        assert_eq!(request.id, 42);
        assert_eq!(request.method, method);
        assert_eq!(request.params, params);
    }

    #[test]
    fn builds_access_key_query() {
        assert_request(
            NearRpc::GetAccountAccessKey {
                address: "account.near".into(),
                public_key: "ed25519:key".into(),
            },
            method::QUERY,
            json!({
                "request_type": "view_access_key",
                "finality": "final",
                "account_id": "account.near",
                "public_key": "ed25519:key"
            }),
        );
    }

    #[test]
    fn builds_function_call_query() {
        assert_request(
            NearRpc::CallFunction {
                contract_id: "token.near".into(),
                method_name: "ft_balance_of".into(),
                args_base64: "encoded-args".into(),
            },
            method::QUERY,
            json!({
                "request_type": "call_function",
                "finality": "final",
                "account_id": "token.near",
                "method_name": "ft_balance_of",
                "args_base64": "encoded-args"
            }),
        );
    }

    #[test]
    fn builds_transaction_status_request() {
        assert_request(
            NearRpc::GetTransactionStatus {
                transaction_hash: "hash".into(),
                sender_account_id: "account.near".into(),
            },
            method::TRANSACTION,
            json!({"tx_hash": "hash", "sender_account_id": "account.near", "wait_until": "EXECUTED"}),
        );
    }

    #[test]
    fn builds_latest_block_request() {
        assert_request(NearRpc::GetLatestBlock, method::BLOCK, json!({"finality": "final"}));
    }

    #[test]
    fn builds_broadcast_request() {
        assert_request(
            NearRpc::SendTransaction {
                signed_transaction: "signed-transaction".into(),
            },
            method::SEND_TRANSACTION,
            json!({"signed_tx_base64": "signed-transaction"}),
        );
    }
}
