use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteQuery {
    pub is_buy_or_sell: String,
    pub fiat_currency: String,
    pub crypto_currency: String,
    pub network: String,
    pub partner_api_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fiat_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crypto_amount: Option<String>,
}
