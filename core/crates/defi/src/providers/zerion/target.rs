use gem_client::{Target, build_path_with_query};

use super::model::PositionsQuery;

#[derive(Clone, Debug)]
pub enum ZerionTarget {
    WalletPositions { address: String, query: PositionsQuery },
}

impl Target for ZerionTarget {
    fn path(&self) -> String {
        match self {
            Self::WalletPositions { address, query } => build_path_with_query(&format!("/v1/wallets/{address}/positions/"), query),
        }
    }
}
