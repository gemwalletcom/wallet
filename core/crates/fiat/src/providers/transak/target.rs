use gem_client::{Target, build_path_with_query};

use super::models::QuoteQuery;

#[derive(Clone, Debug)]
pub enum TransakTarget {
    Quotes { query: QuoteQuery },
    CryptoCurrencies,
    Countries,
    FiatCurrencies,
    RefreshToken { api_key: String },
}

impl Target for TransakTarget {
    fn path(&self) -> String {
        match self {
            Self::Quotes { query } => build_path_with_query("/api/v1/pricing/public/quotes", query),
            Self::CryptoCurrencies => "/cryptocoverage/api/v1/public/crypto-currencies".to_string(),
            Self::Countries => "/api/v2/countries".to_string(),
            Self::FiatCurrencies => "/fiat/public/v1/currencies/fiat-currencies".to_string(),
            Self::RefreshToken { api_key } => build_path_with_query("/partners/api/v2/refresh-token", &[("apiKey", api_key)]),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TransakGatewayTarget {
    AuthSession,
}

impl Target for TransakGatewayTarget {
    fn path(&self) -> String {
        match self {
            Self::AuthSession => "/api/v2/auth/session".to_string(),
        }
    }
}
