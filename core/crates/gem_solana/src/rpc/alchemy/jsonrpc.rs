use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

use crate::COMMITMENT_CONFIRMED;

pub(super) const GET_TRANSACTIONS_FOR_ADDRESS: &str = "getTransactionsForAddress";

pub(super) enum AlchemySolanaRpc {
    GetTransactionsForAddress { address: String, limit: usize },
}

impl ToJsonRpcRequest for AlchemySolanaRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::GetTransactionsForAddress { .. } => GET_TRANSACTIONS_FOR_ADDRESS,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::GetTransactionsForAddress { address, limit } => json!([address, {
                "transactionDetails": "signatures",
                "sortOrder": "desc",
                "limit": limit,
                "commitment": COMMITMENT_CONFIRMED
            }]),
        }
    }
}
