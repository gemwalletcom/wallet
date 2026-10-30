use gem_client::{Target, build_path_with_query};
use primitives::FiatQuoteType;

use super::models::{QuoteQuery, QuoteSellQuery};

#[derive(Clone, Debug)]
pub enum MercuryoTarget {
    BuyRate { query: QuoteQuery },
    Convert { query: QuoteSellQuery },
    Currencies,
    CardCountries,
    CurrencyLimits { from: String, to: String, widget_id: String, quote_type: FiatQuoteType },
}

impl Target for MercuryoTarget {
    fn path(&self) -> String {
        match self {
            Self::BuyRate { query } => build_path_with_query("/v1.6/widget/buy/rate", query),
            Self::Convert { query } => build_path_with_query("/v1.6/public/convert", query),
            Self::Currencies => "/v1.6/lib/currencies".to_string(),
            Self::CardCountries => "/v1.6/public/card-countries?type=alpha2".to_string(),
            Self::CurrencyLimits { from, to, widget_id, quote_type } => build_path_with_query(
                "/v1.6/public/currency-limits",
                &[("from", from.as_str()), ("to", to.as_str()), ("widget_id", widget_id.as_str()), ("type", quote_type.as_ref())],
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path() {
        let target = MercuryoTarget::CurrencyLimits {
            from: "BTC".to_string(),
            to: "USD".to_string(),
            widget_id: "widget".to_string(),
            quote_type: FiatQuoteType::Sell,
        };

        assert_eq!(target.path(), "/v1.6/public/currency-limits?from=BTC&to=USD&widget_id=widget&type=sell");
    }
}
