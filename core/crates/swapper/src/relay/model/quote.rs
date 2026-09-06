use serde::{Deserialize, Serialize};

use super::{EvmStepData, SolanaStepData, TonStepData, TronStepData};
use crate::{SwapperError, error::ProviderErrorResponse};

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
    pub include_compute_unit_limit: bool,
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
pub struct RelayQuoteResponse {
    pub steps: Vec<Step>,
    pub details: QuoteDetails,
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
            StepData::Tron(_) | StepData::Solana(_) | StepData::Ton(_) => None,
        }
    }

    pub fn get_tron_step(&self) -> Option<&TronStepData> {
        match self.step_data()? {
            StepData::Tron(tron) => Some(tron),
            StepData::Evm(_) | StepData::Solana(_) | StepData::Ton(_) => None,
        }
    }

    pub fn get_solana_step(&self) -> Option<&SolanaStepData> {
        match self.step_data()? {
            StepData::Solana(solana) => Some(solana),
            StepData::Evm(_) | StepData::Tron(_) | StepData::Ton(_) => None,
        }
    }

    pub fn get_ton_step(&self) -> Option<&TonStepData> {
        match self.step_data()? {
            StepData::Ton(ton) => Some(ton),
            StepData::Evm(_) | StepData::Tron(_) | StepData::Solana(_) => None,
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
    Solana(SolanaStepData),
    Ton(TonStepData),
}

impl StepData {
    pub fn to_address(&self) -> Option<String> {
        match self {
            Self::Evm(evm) => Some(evm.to.clone()),
            Self::Tron(tron) => Some(tron.trigger_smart_contract()?.contract_address.clone()),
            Self::Solana(_) => None,
            Self::Ton(ton) => Some(ton.messages.first()?.to.clone()),
        }
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

    pub fn eta_in_seconds(&self) -> Option<u32> {
        self.time_estimate.filter(|seconds| *seconds > 0.0).map(|seconds| seconds.ceil() as u32)
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

impl ProviderErrorResponse for RelayErrorResponse {
    fn into_swapper_error(self) -> Option<SwapperError> {
        match self.error_code {
            RelayErrorCode::AmountTooLow => Some(SwapperError::InputAmountError { min_amount: None }),
            RelayErrorCode::NoQuotes | RelayErrorCode::NoSwapRoutesFound => Some(SwapperError::NoQuoteAvailable),
            RelayErrorCode::Unknown => self.message.filter(|message| !message.is_empty()).map(SwapperError::ComputeQuoteError),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_response_skips_approval_step() {
        let response: RelayQuoteResponse = serde_json::from_str(include_str!("../testdata/quote_celo_native_to_bsc_usdt.json")).unwrap();

        assert_eq!(response.steps.len(), 2);
        assert_eq!(response.steps[0].id, "approve");
        assert_eq!(response.router_address(), response.get_evm_step().map(|evm| evm.to.clone()));
        assert_eq!(response.get_evm_step().unwrap().gas_limit_with_buffer().as_deref(), Some("724402"));
    }
}
