use gem_client::{Target, build_path_with_query};

use super::models::{QuoteQuery, QuoteSellQuery};

#[derive(Clone, Debug)]
pub enum MercuryoTarget {
    BuyRate { query: QuoteQuery },
    Convert { query: QuoteSellQuery },
    Currencies,
    CardCountries,
    CurrencyLimits { from: String, to: String, widget_id: String },
}

impl Target for MercuryoTarget {
    fn path(&self) -> String {
        match self {
            Self::BuyRate { query } => build_path_with_query("/v1.6/widget/buy/rate", query),
            Self::Convert { query } => build_path_with_query("/v1.6/public/convert", query),
            Self::Currencies => "/v1.6/lib/currencies".to_string(),
            Self::CardCountries => "/v1.6/public/card-countries?type=alpha2".to_string(),
            Self::CurrencyLimits { from, to, widget_id } => build_path_with_query("/v1.6/public/currency-limits", &[("from", from), ("to", to), ("widget_id", widget_id)]),
        }
    }
}
