use gem_client::{Target, build_path_with_query};

use crate::rpc::trongrid::model::TransactionsQuery;

#[derive(Clone, Debug)]
pub enum TronGridTarget {
    GetTransactions { address: String, query: TransactionsQuery },
    GetTrc20Transactions { address: String, query: TransactionsQuery },
    GetAccount { address: String },
}

impl Target for TronGridTarget {
    fn path(&self) -> String {
        match self {
            Self::GetTransactions { address, query } => build_path_with_query(&format!("/v1/accounts/{address}/transactions"), query),
            Self::GetTrc20Transactions { address, query } => build_path_with_query(&format!("/v1/accounts/{address}/transactions/trc20"), query),
            Self::GetAccount { address } => format!("/v1/accounts/{address}"),
        }
    }
}
