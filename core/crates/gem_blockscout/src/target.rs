use gem_client::{Target, build_path_with_query};

use crate::model::PageQuery;

#[derive(Clone, Debug)]
pub enum BlockscoutTarget {
    Transactions { chain_id: u64, address: String, query: PageQuery },
    TokenTransfers { chain_id: u64, address: String, query: PageQuery },
    TokenBalances { chain_id: u64, address: String },
}

impl Target for BlockscoutTarget {
    fn path(&self) -> String {
        match self {
            Self::Transactions { chain_id, address, query } => build_path_with_query(&format!("/{chain_id}/api/v2/addresses/{address}/transactions"), query),
            Self::TokenTransfers { chain_id, address, query } => build_path_with_query(&format!("/{chain_id}/api/v2/addresses/{address}/token-transfers"), query),
            Self::TokenBalances { chain_id, address } => format!("/{chain_id}/api/v2/addresses/{address}/token-balances"),
        }
    }
}
