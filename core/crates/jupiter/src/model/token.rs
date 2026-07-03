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

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    #[serde(default)]
    pub price_change: f64,
    pub buy_volume: Option<f64>,
    pub sell_volume: Option<f64>,
}
