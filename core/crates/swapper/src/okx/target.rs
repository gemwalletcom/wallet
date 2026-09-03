use gem_client::{Target, build_path_with_query};

use super::model::{QuoteParams, SwapParams};

#[derive(Clone, Debug)]
pub(super) enum OkxTarget {
    Quote { params: QuoteParams },
    Swap { params: SwapParams },
}

impl Target for OkxTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote { params } => build_path_with_query("/api/v6/dex/aggregator/quote", params),
            Self::Swap { params } => build_path_with_query("/api/v6/dex/aggregator/swap", params),
        }
    }
}
