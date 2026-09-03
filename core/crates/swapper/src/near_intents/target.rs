use gem_client::{Target, build_path_with_query};

use super::model::ExplorerTransactionsQuery;

#[derive(Clone, Debug)]
pub enum NearIntentsTarget {
    Quote,
}

impl Target for NearIntentsTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/v0/quote".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum NearIntentsExplorerTarget {
    Transactions { query: ExplorerTransactionsQuery },
}

impl Target for NearIntentsExplorerTarget {
    fn path(&self) -> String {
        match self {
            Self::Transactions { query } => build_path_with_query("/api/v0/transactions", query),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(
            NearIntentsExplorerTarget::Transactions {
                query: ExplorerTransactionsQuery {
                    search: "0xabc".into(),
                    number_of_transactions: 10
                }
            }
            .path(),
            "/api/v0/transactions?search=0xabc&numberOfTransactions=10"
        );
    }
}
