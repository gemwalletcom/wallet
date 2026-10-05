use serde::Deserialize;
use serde_serializers::deserialize_u64_from_str_or_int;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgersResponse {
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub res_code: u64,
    pub res_msg: String,
    pub data: serde_json::Value,
}
