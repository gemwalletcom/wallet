use gem_client::{ContentType, Target};

#[derive(Clone, Debug)]
pub enum AlgorandTarget {
    GetAccount { address: String },
    GetAsset { asset_id: String },
    GetTransactionsParams,
    GetPendingTransaction { transaction_id: String },
    SendTransaction,
}

impl Target for AlgorandTarget {
    fn path(&self) -> String {
        match self {
            Self::GetAccount { address } => format!("/v2/accounts/{address}"),
            Self::GetAsset { asset_id } => format!("/v2/assets/{asset_id}"),
            Self::GetTransactionsParams => "/v2/transactions/params".to_string(),
            Self::GetPendingTransaction { transaction_id } => format!("/v2/transactions/pending/{transaction_id}"),
            Self::SendTransaction => "/v2/transactions".to_string(),
        }
    }

    fn content_type(&self) -> ContentType {
        match self {
            Self::SendTransaction => ContentType::ApplicationXBinary,
            _ => ContentType::ApplicationJson,
        }
    }
}

#[derive(Clone, Debug)]
pub enum AlgorandIndexerTarget {
    AccountTransactions { address: String },
    Block { number: u64 },
    Transaction { id: String },
}

impl Target for AlgorandIndexerTarget {
    fn path(&self) -> String {
        match self {
            Self::AccountTransactions { address } => format!("/v2/accounts/{address}/transactions"),
            Self::Block { number } => format!("/v2/blocks/{number}"),
            Self::Transaction { id } => format!("/v2/transactions/{id}"),
        }
    }
}
