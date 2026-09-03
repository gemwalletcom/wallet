use gem_client::{Target, build_path_with_query};

use super::models::{BuyQuoteQuery, SellQuoteQuery};

#[derive(Clone, Debug)]
pub enum MoonPayTarget {
    IpAddress { ip_address: String },
    BuyQuote { symbol: String, query: BuyQuoteQuery },
    SellQuote { symbol: String, query: SellQuoteQuery },
    Currencies,
    Countries,
}

impl Target for MoonPayTarget {
    fn path(&self) -> String {
        match self {
            Self::IpAddress { ip_address } => build_path_with_query("/v4/ip_address/", &[("ipAddress", ip_address)]),
            Self::BuyQuote { symbol, query } => build_path_with_query(&format!("/v3/currencies/{symbol}/buy_quote/"), query),
            Self::SellQuote { symbol, query } => build_path_with_query(&format!("/v3/currencies/{symbol}/sell_quote/"), query),
            Self::Currencies => "/v3/currencies".to_string(),
            Self::Countries => "/v3/countries".to_string(),
        }
    }
}
