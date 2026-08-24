use gem_jsonrpc::types::ToJsonRpcRequest;
use serde_json::{Value, json};

const GET_ACCOUNT_BALANCE: &str = "ankr_getAccountBalance";
const GET_TOKEN_TRANSFERS: &str = "ankr_getTokenTransfers";
const GET_TRANSACTIONS_BY_ADDRESS: &str = "ankr_getTransactionsByAddress";

pub(super) enum AnkrRpc<'a> {
    AccountBalance { address: &'a str, network: &'static str },
    TokenTransfers { address: &'a str, network: &'static str, limit: usize },
    TransactionsByAddress { address: &'a str, network: &'static str, limit: usize },
}

impl ToJsonRpcRequest for AnkrRpc<'_> {
    fn method(&self) -> &'static str {
        match self {
            Self::AccountBalance { .. } => GET_ACCOUNT_BALANCE,
            Self::TokenTransfers { .. } => GET_TOKEN_TRANSFERS,
            Self::TransactionsByAddress { .. } => GET_TRANSACTIONS_BY_ADDRESS,
        }
    }

    fn params(&self) -> Value {
        match self {
            Self::AccountBalance { address, network } => json!([{
                "walletAddress": address,
                "blockchain": network,
                "onlyWhitelisted": true
            }]),
            Self::TokenTransfers { address, network, limit } => json!({
                "address": address,
                "blockchain": network,
                "pageSize": limit
            }),
            Self::TransactionsByAddress { address, network, limit } => json!({
                "address": address,
                "blockchain": network,
                "pageSize": limit,
                "descOrder": true
            }),
        }
    }
}
