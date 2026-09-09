use crate::{AssetId, FiatQuoteType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteRequest {
    pub asset_id: AssetId,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    pub amount: f64,
    pub currency: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    pub ip_address: String,
}
