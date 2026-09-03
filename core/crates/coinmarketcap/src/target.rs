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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        assert_eq!(CoinMarketCapTarget::LatestListings { limit: 2 }.path(), "/v1/cryptocurrency/listings/latest?limit=2");
        assert_eq!(
            CoinMarketCapTarget::Info {
                key: "id".into(),
                value: "1027,825".into()
            }
            .path(),
            "/v2/cryptocurrency/info?id=1027%2C825"
        );
    }
}
