use std::collections::BTreeSet;

use gem_evm::{address::ethereum_address_checksum, provider::preload_mapper::calculate_gas_limit_with_increase};
use gem_tron::models::{TriggerSmartContractData, TronContractType};
use num_bigint::BigInt;
use primitives::swap::SwapStatus;
use serde::{Deserialize, Serialize};
use serde_serializers::{deserialize_option_bigint_from_str, serialize_option_bigint};

use crate::SwapperError;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_tolerance: Option<String>,
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

    pub fn get_evm_step(&self) -> Option<&EvmStepData> {
        match self.step_data()? {
            StepData::Evm(evm) => Some(evm),
            StepData::Tron(_) => None,
        }
    }

    pub fn get_tron_step(&self) -> Option<&TronStepData> {
        match self.step_data()? {
            StepData::Evm(_) => None,
            StepData::Tron(tron) => Some(tron),
        }
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
        self.step_data()?.to_address()
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
    Tron(TronStepData),
}

impl StepData {
    pub fn to_address(&self) -> Option<String> {
        match self {
            Self::Evm(evm) => Some(evm.to.clone()),
            Self::Tron(tron) => Some(tron.trigger_smart_contract()?.contract_address.clone()),
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
pub struct TronStepData {
    pub parameter: TriggerSmartContractData,
    #[serde(rename = "type")]
    pub transaction_type: TronContractType,
}

impl TronStepData {
    pub fn trigger_smart_contract(&self) -> Option<&TriggerSmartContractData> {
        (self.transaction_type == TronContractType::TriggerSmart).then_some(&self.parameter)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    pub currency_out: CurrencyAmount,
    pub time_estimate: Option<f64>,
    pub slippage_tolerance: Option<RelaySlippageTolerance>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelaySlippageTolerance {
    pub total: String,
}

impl QuoteDetails {
    pub fn slippage_bps(&self) -> Option<u32> {
        self.slippage_tolerance.as_ref()?.total.parse().ok()
    }

    pub fn time_estimate_u32(&self) -> Option<u32> {
        let value = self.time_estimate?;
        if !value.is_finite() || value < 0.0 || value > u32::MAX as f64 {
            return None;
        }
        Some(value.ceil() as u32)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyAmount {
    pub amount: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RelayErrorCode {
    AmountTooLow,
    NoQuotes,
    NoSwapRoutesFound,
    #[default]
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelayErrorResponse {
    #[serde(default)]
    pub error_code: RelayErrorCode,
    pub message: Option<String>,
}

impl RelayErrorResponse {
    pub fn into_swapper_error(self) -> Option<SwapperError> {
        match self.error_code {
            RelayErrorCode::AmountTooLow => Some(SwapperError::InputAmountError { min_amount: None }),
            RelayErrorCode::NoQuotes | RelayErrorCode::NoSwapRoutesFound => Some(SwapperError::NoQuoteAvailable),
            RelayErrorCode::Unknown => self.message.filter(|message| !message.is_empty()).map(SwapperError::ComputeQuoteError),
        }
    }
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

        let evm = response.get_evm_step().unwrap();
        assert_eq!(evm.gas, Some(BigInt::from(482935)));
        assert_eq!(evm.gas_limit_with_buffer().as_deref(), Some("724402"));

        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_usdt_to_base_usdc.json")).unwrap();
        let tron = response.get_tron_step().unwrap();
        let contract = tron.trigger_smart_contract().unwrap();
        assert_eq!(contract.contract_address, "41f0623e1012177482912fb057e44e1a9769b1f588");
        assert_eq!(contract.call_value, None);
        assert_eq!(response.details.currency_out.amount, "976767");

        let response: RelayQuoteResponse = serde_json::from_str(include_str!("testdata/quote_tron_to_base_usdc.json")).unwrap();
        let tron = response.get_tron_step().unwrap();
        assert_eq!(tron.trigger_smart_contract().unwrap().call_value, Some(10_000_000));
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
