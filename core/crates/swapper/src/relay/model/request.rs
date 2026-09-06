use primitives::{decode_hex, swap::SwapStatus};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelayStatus {
    Pending,
    Waiting,
    Depositing,
    Submitted,
    Success,
    Completed,
    Failed,
    Failure,
    Refund,
    Refunded,
    #[serde(other)]
    Unknown,
}

impl RelayStatus {
    pub fn into_swap_status(self) -> SwapStatus {
        match self {
            RelayStatus::Pending | RelayStatus::Waiting | RelayStatus::Depositing | RelayStatus::Submitted | RelayStatus::Unknown => SwapStatus::Pending,
            RelayStatus::Success | RelayStatus::Completed => SwapStatus::Completed,
            RelayStatus::Failed | RelayStatus::Failure | RelayStatus::Refund | RelayStatus::Refunded => SwapStatus::Failed,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestsResponse {
    pub requests: Vec<RelayRequest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequest {
    pub status: RelayStatus,
    pub data: Option<RelayRequestData>,
}

impl RelayRequest {
    pub fn has_input_transaction(&self, transaction_hashes: &[Vec<u8>]) -> bool {
        self.data.as_ref().is_some_and(|data| {
            data.in_txs
                .iter()
                .filter_map(|transaction| decode_hex(transaction.tx_hash.as_deref()?).ok())
                .any(|hash| transaction_hashes.contains(&hash))
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestData {
    pub route: Option<RelayRoute>,
    #[serde(default)]
    pub in_txs: Vec<RelayRequestTransaction>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestTransaction {
    pub tx_hash: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRoute {
    pub actual: Option<RelayRouteActual>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRouteActual {
    pub origin: Option<RelayRouteSide>,
    pub destination: Option<RelayRouteSide>,
}

impl RelayRouteActual {
    pub fn currency_in(&self) -> Option<&RelayCurrencyDetail> {
        self.origin.as_ref()?.input_currency.as_ref()
    }

    pub fn currency_out(&self) -> Option<&RelayCurrencyDetail> {
        let origin_output = self.origin.as_ref().and_then(|origin| origin.output_currency.as_ref());
        self.destination.as_ref().and_then(|destination| destination.output_currency.as_ref()).or(origin_output)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRouteSide {
    pub input_currency: Option<RelayCurrencyDetail>,
    pub output_currency: Option<RelayCurrencyDetail>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayCurrencyDetail {
    pub currency: RelayCurrency,
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayCurrency {
    pub chain_id: u64,
    pub address: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_input_transaction() {
        let response: RelayRequestsResponse = serde_json::from_str(include_str!("../testdata/request_ton_to_robinhood.json")).unwrap();
        let request = &response.requests[0];
        let depository_hash = decode_hex("e7844ac5fd48b3f5dcfbcecb34a8d5cfe2614ea6c4e4dc1cbd5b528db6ab5fac").unwrap();
        let other_hash = decode_hex("99b98b83646fbae04d5b86085f9af9fa785d506a7fcffad85547ab3112866956").unwrap();

        assert!(request.has_input_transaction(&[depository_hash.clone()]));
        assert!(!request.has_input_transaction(&[other_hash]));
        assert!(!RelayRequest::mock_with_status(RelayStatus::Pending).has_input_transaction(&[depository_hash]));
    }

    #[test]
    fn test_relay_status_refund_maps_to_failed() {
        let request: RelayRequest = serde_json::from_value(serde_json::json!({
            "status": "refund",
            "data": null
        }))
        .unwrap();

        assert_eq!(request.status.into_swap_status(), SwapStatus::Failed);
    }
}
