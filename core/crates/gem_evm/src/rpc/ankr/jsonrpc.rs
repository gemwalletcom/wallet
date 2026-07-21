use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::json;

use crate::method;

#[derive(Clone, Debug)]
pub(super) enum AnkrRpc {
    AccountBalance { address: String, chain: &'static str },
    TokenTransfers { address: String, chain: &'static str, limit: usize },
    TransactionsByAddress { address: String, chain: &'static str, limit: usize },
}

impl ToJsonRpcRequest for AnkrRpc {
    fn method(&self) -> &'static str {
        match self {
            Self::AccountBalance { .. } => method::ANKR_GET_ACCOUNT_BALANCE,
            Self::TokenTransfers { .. } => method::ANKR_GET_TOKEN_TRANSFERS,
            Self::TransactionsByAddress { .. } => method::ANKR_GET_TRANSACTIONS_BY_ADDRESS,
        }
    }

    fn params(&self) -> serde_json::Value {
        match self {
            Self::AccountBalance { address, chain } => json!([{
                "walletAddress": address,
                "blockchain": chain,
                "onlyWhitelisted": true
            }]),
            Self::TokenTransfers { address, chain, limit } => json!({
                "address": address,
                "blockchain": chain,
                "pageSize": limit
            }),
            Self::TransactionsByAddress { address, chain, limit } => json!({
                "address": address,
                "blockchain": chain,
                "pageSize": limit,
                "descOrder": true
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_account_balance_request() {
        let request = AnkrRpc::AccountBalance {
            address: "0x1234".into(),
            chain: "eth",
        }
        .to_jsonrpc_request(7);

        assert_eq!(request.method, method::ANKR_GET_ACCOUNT_BALANCE);
        assert_eq!(request.params, json!([{"walletAddress": "0x1234", "blockchain": "eth", "onlyWhitelisted": true}]));
    }
}
