use primitives::{SimulationPayloadField, SimulationResult};

pub type GemSimulationResult = SimulationResult;
pub type GemSimulationPayloadField = SimulationPayloadField;

uniffi::custom_type!(SimulationResult, String, {
    remote,
    lower: |value| serde_json::to_string(&value).unwrap_or_default(),
    try_lift: |value| serde_json::from_str(&value).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid SimulationResult")),
});

uniffi::custom_type!(SimulationPayloadField, String, {
    remote,
    lower: |value| serde_json::to_string(&value).unwrap_or_default(),
    try_lift: |value| serde_json::from_str(&value).map_err(|_| uniffi::deps::anyhow::Error::msg("Invalid SimulationPayloadField")),
});
