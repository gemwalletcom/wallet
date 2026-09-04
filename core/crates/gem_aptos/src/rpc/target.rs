use gem_client::{ContentType, Target, build_path_with_query};

use crate::models::SimulateTransactionQuery;

#[derive(Clone, Debug)]
pub enum AptosTarget {
    GetLedger,
    GetBlock { height: u64 },
    GetAccount { address: String },
    GetAccountTransactions { address: String },
    GetAccountResource { address: String, resource: String },
    GetAccountBalance { address: String, asset_type: String },
    GetTransaction { hash: String },
    GetGasPrice,
    SimulateTransaction { query: SimulateTransactionQuery },
    SubmitTransaction,
    View,
}

impl Target for AptosTarget {
    fn path(&self) -> String {
        match self {
            Self::GetLedger => "/v1/".to_string(),
            Self::GetBlock { height } => format!("/v1/blocks/by_height/{height}?with_transactions=true"),
            Self::GetAccount { address } => format!("/v1/accounts/{address}"),
            Self::GetAccountTransactions { address } => format!("/v1/accounts/{address}/transactions"),
            Self::GetAccountResource { address, resource } => format!("/v1/accounts/{address}/resource/{resource}"),
            Self::GetAccountBalance { address, asset_type } => format!("/v1/accounts/{address}/balance/{asset_type}"),
            Self::GetTransaction { hash } => format!("/v1/transactions/by_hash/{hash}"),
            Self::GetGasPrice => "/v1/estimate_gas_price".to_string(),
            Self::SimulateTransaction { query } => build_path_with_query("/v1/transactions/simulate", query),
            Self::SubmitTransaction => "/v1/transactions".to_string(),
            Self::View => "/v1/view".to_string(),
        }
    }

    fn content_type(&self) -> ContentType {
        match self {
            Self::SubmitTransaction => ContentType::ApplicationAptosBcs,
            _ => ContentType::ApplicationJson,
        }
    }
}
