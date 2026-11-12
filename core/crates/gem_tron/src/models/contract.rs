use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct TronContractRequest {
    pub value: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct TronContractInfo {
    #[serde(default)]
    pub origin_address: String,
    #[serde(default)]
    pub consume_user_resource_percent: u64,
    #[serde(default)]
    pub origin_energy_limit: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerSmartContractData {
    pub contract_address: String,
    pub data: String,
    pub owner_address: String,
    pub fee_limit: Option<u64>,
    pub call_value: Option<u64>,
}
