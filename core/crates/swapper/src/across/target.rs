use gem_client::{Target, build_path_with_query};

use super::model::AcrossDepositsQuery;

#[derive(Clone, Debug)]
pub enum AcrossTarget {
    Deposits { query: AcrossDepositsQuery },
}

impl Target for AcrossTarget {
    fn path(&self) -> String {
        match self {
            Self::Deposits { query } => build_path_with_query("/deposits", query),
        }
    }
}
