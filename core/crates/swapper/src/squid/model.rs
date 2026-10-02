use primitives::swap::{SlippageMode, SwapStatus};
use serde::{Deserialize, Serialize};

use crate::{SwapperError, SwapperSlippage, fees::percent_to_bps};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidRouteRequest {
    pub from_chain: String,
    pub to_chain: String,
    pub from_token: String,
    pub to_token: String,
    pub from_amount: String,
    pub from_address: String,
    pub to_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_config: Option<SlippageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    pub quote_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SlippageConfig {
    pub auto_mode: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SquidRouteResponse {
    pub route: SquidRoute,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidRoute {
    pub estimate: SquidEstimate,
    #[serde(deserialize_with = "deserialize_transaction_request")]
    pub transaction_request: Option<SquidTransactionRequest>,
}

fn deserialize_transaction_request<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Option<SquidTransactionRequest>, D::Error> {
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        Some(v) if v.as_object().is_some_and(|m| m.contains_key("data")) => serde_json::from_value(v).map(Some).map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidEstimate {
    pub to_amount: String,
    pub estimated_route_duration: u32,
    pub aggregate_slippage: f64,
}

impl SquidEstimate {
    pub fn slippage_bps(&self, requested: &SwapperSlippage) -> Result<u32, SwapperError> {
        match requested.mode {
            SlippageMode::Exact => Ok(requested.bps),
            SlippageMode::Auto => percent_to_bps(self.aggregate_slippage).ok_or(SwapperError::InvalidRoute),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidTransactionRequest {
    pub target: String,
    pub data: String,
    pub value: String,
    pub gas_limit: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SquidStatusResponse {
    pub squid_transaction_status: SquidStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SquidStatus {
    Success,
    Ongoing,
    PartialSuccess,
    NeedsGas,
    NotFound,
    Refund,
}

impl SquidStatus {
    pub fn swap_status(&self) -> SwapStatus {
        match self {
            Self::Success | Self::PartialSuccess => SwapStatus::Completed,
            Self::Ongoing | Self::NeedsGas | Self::NotFound => SwapStatus::Pending,
            Self::Refund => SwapStatus::Refunded,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estimate_slippage_bps() {
        let estimate = serde_json::from_str::<SquidRouteResponse>(include_str!("../../testdata/squid/route_osmosis_to_cosmos_auto.json")).unwrap().route.estimate;

        assert_eq!(estimate.slippage_bps(&SwapperSlippage { bps: 100, mode: SlippageMode::Auto }), Ok(50), "auto reports the slippage Squid picked");
        assert_eq!(estimate.slippage_bps(&SwapperSlippage { bps: 100, mode: SlippageMode::Exact }), Ok(100), "a chosen slippage stays the one asked for");
    }

    #[test]
    fn test_deserialize_status_response() {
        let result: SquidStatusResponse = serde_json::from_str(include_str!("../../testdata/squid/status_response.json")).unwrap();
        assert_eq!(result.squid_transaction_status, SquidStatus::Success);
        assert_eq!(result.squid_transaction_status.swap_status(), SwapStatus::Completed);
    }
}
