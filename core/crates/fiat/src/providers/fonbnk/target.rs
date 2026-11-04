use gem_client::{Target, build_path_with_query};

use super::model::FonbnkOrderLimitsQuery;

#[derive(Clone, Debug)]
pub enum FonbnkTarget {
    Currencies,
    OrderLimits { query: FonbnkOrderLimitsQuery },
    Quote,
}

impl Target for FonbnkTarget {
    fn path(&self) -> String {
        match self {
            Self::Currencies => "/api/v2/currencies".to_string(),
            Self::OrderLimits { query } => build_path_with_query("/api/v2/order-limits", query),
            Self::Quote => "/api/v2/quote".to_string(),
        }
    }
}
