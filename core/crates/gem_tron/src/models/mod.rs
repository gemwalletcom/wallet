use primitives::hex::decode_hex_utf8;
use serde::{Deserialize, Serialize};
use std::{error::Error, fmt};

pub mod account;
pub mod block;
pub mod contract;
pub mod contract_type;
#[cfg(feature = "signer")]
pub(crate) mod signing;
pub mod transaction;

pub use account::*;
pub use block::*;
pub use contract::*;
pub use contract_type::*;
#[cfg(feature = "signer")]
pub(crate) use signing::*;
pub use transaction::*;

#[derive(Debug, Deserialize, Serialize)]
pub struct Block {
    pub block_header: BlockHeader,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockId {
    #[serde(rename = "blockID")]
    pub block_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockTransactions {
    pub block_header: BlockHeader,
    #[serde(default)]
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeader {
    pub raw_data: BlockHeaderData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeaderData {
    pub number: i64,
}

#[derive(Serialize, Debug)]
pub struct TriggerConstantContractRequest {
    pub owner_address: String,
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_value: Option<u64>,
    pub visible: bool,
}

#[derive(Deserialize, Debug)]
pub struct TriggerConstantContractResponse {
    #[serde(default)]
    pub constant_result: Vec<String>,
    pub result: Option<TriggerContractResult>,
    pub energy_used: Option<u64>,
    pub energy_penalty: Option<u64>,
    #[serde(default)]
    pub logs: Option<Vec<TronLog>>,
}

impl TriggerConstantContractResponse {
    /// Returns `energy_used` (the total, which already includes `energy_penalty` — don't add it) and surfaces failed simulations as errors.
    pub fn get_energy(&self) -> Result<u64, TronRpcError> {
        if let Some(error) = self.result.as_ref().and_then(|r| r.check_error()) {
            return Err(error);
        }
        // Some reverts still report result=true; the failure only shows up in the message
        if let Some(message) = self.result.as_ref().and_then(|r| r.message.as_deref()) {
            return Err(TronRpcError {
                code: None,
                message: Some(decode_hex_utf8(message).unwrap_or_else(|| message.to_string())),
            });
        }
        self.energy_used.ok_or_else(|| TronRpcError {
            code: None,
            message: Some("Tron triggerconstantcontract response missing energy_used".to_string()),
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct TriggerContractResult {
    pub result: Option<bool>,
    pub code: Option<String>,
    pub message: Option<String>,
}

impl TriggerContractResult {
    pub fn check_error(&self) -> Option<TronRpcError> {
        if self.result.unwrap_or(false) {
            return None;
        }

        let message = self
            .message
            .as_deref()
            .map(|message_hex| decode_hex_utf8(message_hex).unwrap_or_else(|| message_hex.to_string()));

        Some(TronRpcError { code: self.code.clone(), message })
    }
}

#[derive(Debug, Clone)]
pub struct TronRpcError {
    pub code: Option<String>,
    pub message: Option<String>,
}

impl fmt::Display for TronRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tron RPC Error {} {}", self.code.as_deref().unwrap_or(""), self.message.as_deref().unwrap_or(""))
    }
}

impl Error for TronRpcError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessesList {
    pub witnesses: Vec<WitnessAccount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WitnessAccount {
    pub address: String,
    pub vote_count: Option<i64>,
    pub url: String,
    pub is_jobs: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainParametersResponse {
    pub chain_parameter: Vec<ChainParameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParameter {
    pub key: String,
    pub value: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::address::TronAddress;

    #[test]
    fn test_get_energy_reverted_with_success_flag_and_message() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_reverted.json")).unwrap();

        let error = response.get_energy().unwrap_err();
        assert_eq!(error.message.as_deref(), Some("REVERT opcode executed"));
    }

    #[test]
    fn test_get_energy_success_with_logs() {
        let response: TriggerConstantContractResponse = serde_json::from_str(include_str!("../../testdata/trigger_constant_contract_with_transfer_log.json")).unwrap();

        assert_eq!(response.get_energy().unwrap(), 173171);
        let logs = response.logs.as_ref().unwrap();
        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].address, TronAddress::from_hex_or_base58("88zffnwSJQ1BGNdvoHNCr24pGHX9ZHmWw"));
    }
}
