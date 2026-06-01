use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectResult {
    pub effects: InspectEffects,
    pub events: serde_json::Value,
    pub error: Option<String>,
    #[serde(default)]
    pub results: Vec<InspectCommandResult>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectCommandResult {
    #[serde(default)]
    pub return_values: Vec<InspectReturnValue>,
}

pub type InspectReturnValue = (Vec<u8>, String);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectEvent<T> {
    pub package_id: String,
    pub transaction_module: String,
    pub parsed_json: T,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectEffects {
    pub gas_used: InspectGasUsed,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectGasUsed {
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub computation_cost: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub storage_cost: u64,
    #[serde(deserialize_with = "deserialize_u64_from_str")]
    pub storage_rebate: u64,
}
