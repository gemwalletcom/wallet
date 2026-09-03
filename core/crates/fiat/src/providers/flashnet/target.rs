use gem_client::{Target, build_path_with_query};

use super::model::EstimateQuery;

#[derive(Clone, Debug)]
pub enum FlashnetTarget {
    Routes,
    Onramp,
    Estimate { query: EstimateQuery },
}

impl Target for FlashnetTarget {
    fn path(&self) -> String {
        match self {
            Self::Routes => "/v1/orchestration/routes".to_string(),
            Self::Onramp => "/v1/orchestration/onramp".to_string(),
            Self::Estimate { query } => build_path_with_query("/v1/orchestration/estimate", query),
        }
    }
}
