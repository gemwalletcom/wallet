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
