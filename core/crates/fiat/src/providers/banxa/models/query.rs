use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyQuoteQuery {
    pub payment_method_id: &'static str,
    pub crypto: String,
    pub blockchain: String,
    pub fiat: String,
    pub fiat_amount: String,
}
