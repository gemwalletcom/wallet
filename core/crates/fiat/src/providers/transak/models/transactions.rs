use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransakOrderResponse {
    #[serde(alias = "orderId")]
    pub id: String,
    pub partner_order_id: Option<String>,
    pub quote_id: Option<String>,
    pub status: String,
    pub fiat_currency: String,
    pub fiat_amount: f64,
    pub transaction_hash: Option<String>,
}
