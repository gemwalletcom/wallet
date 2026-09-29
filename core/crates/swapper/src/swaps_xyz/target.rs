use gem_client::{Target, build_path_with_query};

use super::model::{PathsQuery, StatusQuery, TransactionsQuery};

#[derive(Clone, Debug)]
pub(super) enum SwapsXyzTarget {
    Paths { query: PathsQuery },
    Action,
    Status { query: StatusQuery },
    Transactions { query: TransactionsQuery },
}

impl Target for SwapsXyzTarget {
    fn path(&self) -> String {
        match self {
            Self::Paths { query } => build_path_with_query("/getPaths", query),
            Self::Action => "/action".to_string(),
            Self::Status { query } => build_path_with_query("/getStatus", query),
            Self::Transactions { query } => build_path_with_query("/getTransactions", query),
        }
    }
}
