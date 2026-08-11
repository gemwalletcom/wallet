use std::collections::BTreeSet;

use gem_evm::{address::ethereum_address_checksum, provider::preload_mapper::calculate_gas_limit_with_increase};
use num_bigint::BigInt;
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_bigint_from_str, serialize_option_bigint};

const STEP_SWAP: &str = "swap";
const STEP_DEPOSIT: &str = "deposit";
const STEP_APPROVE: &str = "approve";
const STEP_TRANSACTION: &str = "transaction";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayQuoteRequest {
    pub user: String,
    pub origin_chain_id: u64,
    pub destination_chain_id: u64,
    pub origin_currency: String,
    pub destination_currency: String,
    pub amount: String,
    pub recipient: String,
    pub trade_type: String,
    pub refund_to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub app_fees: Vec<RelayAppFee>,
    pub max_route_length: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelayAppFee {
    pub recipient: String,
    pub fee: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayFees {
    pub gas: Option<RelayFeeAmount>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayFeeAmount {
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayQuoteResponse {
    pub steps: Vec<Step>,
    pub details: QuoteDetails,
    pub fees: Option<RelayFees>,
}

impl RelayQuoteResponse {
    pub fn step_data(&self) -> Option<&StepData> {
        self.steps
            .iter()
            .find(|step| step.id == STEP_SWAP || step.id == STEP_DEPOSIT)
            .or_else(|| self.steps.iter().find(|step| step.kind == STEP_TRANSACTION && step.id != STEP_APPROVE))
            .or_else(|| self.steps.iter().find(|step| step.step_data().is_some()))
            .and_then(Step::step_data)
    }

    pub fn router_address(&self) -> Option<String> {
        self.steps.iter().filter(|step| step.id != STEP_APPROVE).find_map(Step::to_address)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    pub id: String,
    pub kind: String,
    pub items: Option<Vec<StepItem>>,
}

impl Step {
    pub fn step_data(&self) -> Option<&StepData> {
        self.items.as_ref()?.first()?.data.as_ref()
    }

    pub fn to_address(&self) -> Option<String> {
        Some(self.step_data()?.to_address())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StepItem {
    pub data: Option<StepData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StepData {
    Evm(EvmStepData),
}

impl StepData {
    pub fn to_address(&self) -> String {
        match self {
            Self::Evm(evm) => evm.to.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvmStepData {
    pub to: String,
    pub data: Option<String>,
    pub value: String,
    #[serde(default, serialize_with = "serialize_option_bigint", deserialize_with = "deserialize_option_bigint_from_str")]
    pub gas: Option<BigInt>,
}

impl EvmStepData {
    pub fn gas_limit_with_buffer(&self) -> Option<String> {
        let gas = self.gas.clone()?;
        if gas <= BigInt::from(0) {
            return None;
        }
        Some(calculate_gas_limit_with_increase(gas).to_string())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_out: CurrencyAmount,
    pub time_estimate: Option<f64>,
    pub swap_impact: Option<SwapImpact>,
}

impl QuoteDetails {
    pub fn time_estimate_u32(&self) -> Option<u32> {
        let value = self.time_estimate?;
        if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
            return None;
        }
        Some(value.ceil() as u32)
    }

    pub fn slippage_bps(&self) -> Option<u32> {
        let percent: f64 = self.swap_impact.as_ref()?.percent.as_ref()?.parse().ok()?;
        Some((percent.abs() * 100.0) as u32)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapImpact {
    pub percent: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAmount {
    pub amount: String,
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayRequestData {
    pub route: Option<RelayRoute>,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainsResponse {
    pub chains: Vec<RelayChainInfo>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayChainInfo {
    #[serde(default)]
    pub solver_addresses: Vec<String>,
    pub protocol: Option<RelayProtocol>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayProtocol {
    pub v2: Option<RelayProtocolV2>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayProtocolV2 {
    pub depository: Option<String>,
}

impl RelayChainsResponse {
    pub fn deposit_addresses(&self) -> Vec<String> {
        self.chains
            .iter()
            .filter_map(|chain| chain.protocol.as_ref()?.v2.as_ref()?.depository.as_ref())
            .map(|address| ethereum_address_checksum(address).unwrap_or_else(|_| address.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn send_addresses(&self) -> Vec<String> {
        self.chains
            .iter()
            .flat_map(|chain| chain.solver_addresses.iter())
            .map(|address| ethereum_address_checksum(address).unwrap_or_else(|_| address.clone()))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_response_deserialization() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_celo_native_to_bsc_usdt.json")).unwrap();

        assert_eq!(response.steps.len(), 2);
        assert_eq!(response.steps[0].id, "approve");

        let StepData::Evm(evm) = response.step_data().unwrap();
        assert_eq!(evm.gas, Some(BigInt::from(482935)));
        assert_eq!(evm.gas_limit_with_buffer().as_deref(), Some("724402"));
    }

    #[test]
    fn test_deposit_addresses() {
        let depository = "0x4cd00e387622c35bddb9b4c962c136462338bc31";
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo {
                    solver_addresses: vec![],
                    protocol: Some(RelayProtocol {
                        v2: Some(RelayProtocolV2 {
                            depository: Some(depository.to_string()),
                        }),
                    }),
                },
                RelayChainInfo {
                    solver_addresses: vec![],
                    protocol: Some(RelayProtocol {
                        v2: Some(RelayProtocolV2 {
                            depository: Some("0x59916da825d2d2ec1bf878d71c88826f6633ecca".to_string()),
                        }),
                    }),
                },
            ],
        };

        assert_eq!(
            response.deposit_addresses(),
            vec![
                ethereum_address_checksum(depository).unwrap(),
                ethereum_address_checksum("0x59916da825d2d2ec1bf878d71c88826f6633ecca").unwrap(),
            ]
        );
    }

    #[test]
    fn test_send_addresses() {
        let solver = "0xf70da97812cb96acdf810712aa562db8dfa3dbef";
        let response = RelayChainsResponse {
            chains: vec![RelayChainInfo {
                solver_addresses: vec![solver.to_string(), solver.to_string()],
                protocol: None,
            }],
        };

        assert_eq!(response.send_addresses(), vec![ethereum_address_checksum(solver).unwrap()]);
    }

    #[test]
    fn test_deposit_addresses_skips_missing_depository() {
        let depository = "0x4cd00e387622c35bddb9b4c962c136462338bc31";
        let response = RelayChainsResponse {
            chains: vec![
                RelayChainInfo {
                    solver_addresses: vec![],
                    protocol: Some(RelayProtocol {
                        v2: Some(RelayProtocolV2 {
                            depository: Some(depository.to_string()),
                        }),
                    }),
                },
                RelayChainInfo {
                    solver_addresses: vec![],
                    protocol: Some(RelayProtocol {
                        v2: Some(RelayProtocolV2 { depository: None }),
                    }),
                },
            ],
        };

        assert_eq!(response.deposit_addresses(), vec![ethereum_address_checksum(depository).unwrap()]);
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
