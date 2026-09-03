use gem_client::{Target, build_path_with_query};

#[derive(Clone, Debug)]
pub enum PythTarget {
    PriceFeeds,
    LatestPrices { ids: Vec<String> },
}

impl Target for PythTarget {
    fn path(&self) -> String {
        match self {
            Self::PriceFeeds => "/v2/price_feeds".to_string(),
            Self::LatestPrices { ids } => build_path_with_query("/v2/updates/price/latest", &ids.iter().map(|id| ("ids[]", id)).collect::<Vec<_>>()),
        }
    }
}
