use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuyQuoteQuery {
    pub base_currency_code: String,
    pub base_currency_amount: String,
    pub are_fees_included: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SellQuoteQuery {
    pub quote_currency_code: String,
    pub quote_currency_amount: String,
    pub are_fees_included: bool,
}
