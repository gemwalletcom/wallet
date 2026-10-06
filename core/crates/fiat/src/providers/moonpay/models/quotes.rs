use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPayBuyQuote {
    pub quote_currency_amount: f64,
    pub total_amount: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoonPaySellQuote {
    pub base_currency_amount: f64,
    pub quote_currency_amount: f64,
}
