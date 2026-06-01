use serde::{Deserialize, Serialize};
use std::error::Error;

use crate::models::{TransactionData, TronContractType};

#[derive(Deserialize)]
struct TriggerSmartContractPayload {
    address: Option<String>,
    transaction: TriggerSmartContractPayloadTransaction,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum TriggerSmartContractPayloadTransaction {
    Direct { raw_data: TransactionData },
    Nested { transaction: TriggerSmartContractNestedTransaction },
}

#[derive(Deserialize)]
struct TriggerSmartContractNestedTransaction {
    raw_data: TransactionData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractCall {
    pub contract_address: String,
    pub function_selector: String,
    pub parameter: Option<String>,
    pub fee_limit: Option<u32>,
    pub call_value: Option<u32>,
    pub owner_address: String,
    pub visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResult {
    pub result: TronSmartContractResultMessage,
    pub constant_result: Vec<String>,
    pub energy_used: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TronSmartContractResultMessage {
    pub result: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSmartContractData {
    pub contract_address: String,
    pub data: String,
    pub owner_address: String,
    pub fee_limit: Option<u64>,
    pub call_value: Option<u64>,
}

impl TriggerSmartContractData {
    pub fn from_payload(data: Option<&[u8]>, sender_address: &str) -> Result<Option<Self>, Box<dyn Error + Send + Sync>> {
        let Some(data) = data else {
            return Ok(None);
        };
        let Ok(payload) = serde_json::from_slice::<TriggerSmartContractPayload>(data) else {
            return Ok(None);
        };
        let raw_data = match payload.transaction {
            TriggerSmartContractPayloadTransaction::Direct { raw_data } => raw_data,
            TriggerSmartContractPayloadTransaction::Nested { transaction } => transaction.raw_data,
        };
        let fee_limit = raw_data.fee_limit;
        let Some(contract) = raw_data.contract.into_iter().next() else {
            return Ok(None);
        };
        if contract.contract_type != Some(TronContractType::TriggerSmart) {
            return Ok(None);
        }

        let value = contract.parameter.value;
        let Some(contract_address) = value.contract_address else {
            return Err("Invalid Tron contract address".into());
        };
        let Some(data) = value.data else {
            return Ok(None);
        };
        let owner_address = payload
            .address
            .filter(|address| !address.is_empty())
            .or(value.owner_address)
            .unwrap_or_else(|| sender_address.to_string());

        Ok(Some(Self {
            contract_address,
            data,
            owner_address,
            fee_limit,
            call_value: value.call_value,
        }))
    }
}
