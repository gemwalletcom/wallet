use gem_client::{Target, build_path_with_query};

use crate::model::TransactionsQuery;

#[derive(Clone, Debug)]
pub enum HeliusTarget {
    AddressTransactions { address: String, query: TransactionsQuery },
}

impl Target for HeliusTarget {
    fn path(&self) -> String {
        match self {
            Self::AddressTransactions { address, query } => build_path_with_query(&format!("/v0/addresses/{address}/transactions"), query),
        }
    }
}
