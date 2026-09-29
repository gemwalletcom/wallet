use serde::{Deserialize, Serialize};

use super::{RelayCurrency, RelayStatus};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerRequestsResponse {
    pub requests: Vec<RelayPartnerRequest>,
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerRequest {
    pub id: String,
    pub status: RelayStatus,
    pub user: String,
    pub recipient: String,
    pub data: RelayPartnerRequestData,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerRequestData {
    pub metadata: Option<RelayPartnerMetadata>,
    #[serde(default)]
    pub paid_app_fees: Vec<RelayPartnerAmount>,
    pub app_fee_currency_object: Option<RelayCurrency>,
    #[serde(default)]
    pub in_txs: Vec<RelayPartnerTransaction>,
    #[serde(default)]
    pub out_txs: Vec<RelayPartnerTransaction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerMetadata {
    pub currency_in: RelayPartnerCurrencyAmount,
    pub currency_out: RelayPartnerCurrencyAmount,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerCurrencyAmount {
    pub currency: RelayCurrency,
    pub amount: String,
    pub amount_usd: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerAmount {
    pub amount: String,
    pub bps: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RelayPartnerTransaction {
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerCursor {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayPartnerRequestsQuery {
    pub referrer: String,
    pub sort_by: &'static str,
    pub sort_direction: &'static str,
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
}
