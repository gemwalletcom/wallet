use gem_client::{Target, build_path_with_query};

use super::model::{ExplorerPartnerTransactionsQuery, ExplorerTransactionsQuery};

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
    PartnerTransactions { query: ExplorerPartnerTransactionsQuery },
}

impl Target for NearIntentsExplorerTarget {
    fn path(&self) -> String {
        match self {
            Self::Transactions { query } => build_path_with_query("/api/v0/transactions", query),
            Self::PartnerTransactions { query } => build_path_with_query("/api/v0/transactions", query),
        }
    }
}
