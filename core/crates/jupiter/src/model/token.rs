use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub icon: Option<String>,
    #[serde(rename = "isVerified")]
    pub is_verified: Option<bool>,
    #[serde(default)]
    pub usd_price: f64,
    pub mcap: Option<f64>,
    pub fdv: Option<f64>,
    pub circ_supply: Option<f64>,
    pub total_supply: Option<f64>,
    #[serde(default)]
    pub stats24h: TokenStats,
}

impl Token {
    pub fn is_verified(&self) -> bool {
        self.is_verified.unwrap_or(false)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenSearchResult {
    pub id: String,
    pub is_verified: Option<bool>,
    pub audit: Option<TokenAudit>,
}

impl TokenSearchResult {
    pub fn is_suspicious(&self) -> bool {
        self.audit.as_ref().and_then(|audit| audit.is_sus).is_some()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TokenAudit {
    pub is_sus: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    #[serde(default)]
    pub price_change: f64,
    pub buy_volume: Option<f64>,
    pub sell_volume: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::TOKEN_SEARCH_RESULTS;

    #[test]
    fn test_token_search_result_status_fields() {
        let results: Vec<TokenSearchResult> = serde_json::from_str(TOKEN_SEARCH_RESULTS).unwrap();

        assert_eq!(results[0].is_verified, Some(true));
        assert_eq!(results[1].is_verified, Some(false));
        assert_eq!(results[2].is_verified, None);
        assert!(!results[0].is_suspicious());
        assert!(!results[1].is_suspicious());
        assert!(results[2].is_suspicious());
    }
}
