use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::json;

use crate::method;

#[derive(Clone, Debug)]
pub enum XrpRpc {
    GetAccountInfo(String),
    GetAccountObjects(String),
    GetAccountTransactions { address: String, limit: usize },
    GetFees,
    GetLedger(u64),
    GetLatestValidatedLedger,
    GetServerInfo,
    GetTransaction(String),
    SubmitTransaction(String),
}

impl ToJsonRpcRequest for XrpRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::GetAccountInfo(_) => method::ACCOUNT_INFO,
            Self::GetAccountObjects(_) => method::ACCOUNT_OBJECTS,
            Self::GetAccountTransactions { .. } => method::ACCOUNT_TRANSACTIONS,
            Self::GetFees => method::FEE,
            Self::GetLedger(_) | Self::GetLatestValidatedLedger => method::LEDGER,
            Self::GetServerInfo => method::SERVER_INFO,
            Self::GetTransaction(_) => method::TRANSACTION,
            Self::SubmitTransaction(_) => method::SUBMIT,
        }
    }

    fn params(&self) -> serde_json::Value {
        match self {
            Self::GetAccountInfo(address) => json!([{
                "account": address,
                "ledger_index": "current"
            }]),
            Self::GetAccountObjects(address) => json!([{
                "account": address,
                "type": "state",
                "ledger_index": "validated"
            }]),
            Self::GetAccountTransactions { address, limit } => json!([{
                "account": address,
                "api_version": 2,
                "limit": limit,
                "ledger_index_max": -1,
                "ledger_index_min": -1
            }]),
            Self::GetFees => json!([{}]),
            Self::GetLedger(block_number) => json!([{
                "ledger_index": block_number,
                "transactions": true,
                "expand": true
            }]),
            Self::GetLatestValidatedLedger => json!([{"ledger_index": "validated"}]),
            Self::GetServerInfo => json!([{}]),
            Self::GetTransaction(transaction_id) => json!([{"transaction": transaction_id}]),
            Self::SubmitTransaction(data) => json!([{
                "tx_blob": data,
                "fail_hard": true
            }]),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    fn assert_request(rpc: XrpRpc, method: &str, params: serde_json::Value) {
        let request = rpc.to_jsonrpc_request(42);
        assert_eq!(request.id, 42);
        assert_eq!(request.method, method);
        assert_eq!(request.params, params);
    }

    #[test]
    fn builds_account_transactions_request() {
        let request = XrpRpc::GetAccountTransactions {
            address: "rAddress".into(),
            limit: 25,
        }
        .to_jsonrpc_request(42);
        let expected: Value = serde_json::from_str(include_str!("../testdata/account_transactions_request.json")).unwrap();

        assert_eq!(serde_json::to_value(request).unwrap(), expected);
    }

    #[test]
    fn builds_broadcast_request_with_fail_hard() {
        assert_request(
            XrpRpc::SubmitTransaction("signed-transaction".into()),
            method::SUBMIT,
            json!([{"tx_blob": "signed-transaction", "fail_hard": true}]),
        );
    }

    #[test]
    fn builds_latest_validated_ledger_request() {
        assert_request(XrpRpc::GetLatestValidatedLedger, method::LEDGER, json!([{"ledger_index": "validated"}]));
    }
}
