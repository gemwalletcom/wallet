use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum CoinMarketCapTarget {
    LatestListings { limit: usize },
    TrendingListings { limit: usize },
    Info { key: String, value: String },
}

impl Target for CoinMarketCapTarget {
    fn path(&self) -> String {
        match self {
            Self::LatestListings { limit } => format!("/v1/cryptocurrency/listings/latest?limit={limit}"),
            Self::TrendingListings { limit } => format!("/v1/cryptocurrency/trending/latest?limit={limit}"),
            Self::Info { key, value } => build_path_with_query("/v2/cryptocurrency/info", &[(key, value)]),
        }
    }
}
