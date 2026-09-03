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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            BlockscoutTarget::Transactions {
                chain_id: 1,
                address: "0x1".into(),
                query: PageQuery::newest(3)
            }
            .path(),
            "/1/api/v2/addresses/0x1/transactions?sort=block_number&order=desc&items_count=3"
        );
        assert_eq!(
            BlockscoutTarget::TokenBalances {
                chain_id: 8453,
                address: "0x1".into()
            }
            .path(),
            "/8453/api/v2/addresses/0x1/token-balances"
        );
    }
}
