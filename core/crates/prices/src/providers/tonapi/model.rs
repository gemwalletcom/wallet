use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RatesResponse {
    pub rates: HashMap<String, TokenRates>,
}

#[derive(Debug, Deserialize)]
pub struct TokenRates {
    pub prices: HashMap<String, f64>,
    pub diff_24h: HashMap<String, String>,
}
