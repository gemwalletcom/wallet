use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum JupiterTarget {
    VerifiedTokens,
    TopTrending { interval: String, limit: usize },
    Search { query: String },
    Positions { address: String },
}

impl Target for JupiterTarget {
    fn path(&self) -> String {
        match self {
            Self::VerifiedTokens => "/tokens/v2/tag?query=verified".to_string(),
            Self::TopTrending { interval, limit } => build_path_with_query(&format!("/tokens/v2/toptrending/{interval}"), &[("limit", limit)]),
            Self::Search { query } => build_path_with_query("/tokens/v2/search", &[("query", query)]),
            Self::Positions { address } => format!("/portfolio/v1/positions/{address}"),
        }
    }
}
