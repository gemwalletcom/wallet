use serde::{Deserialize, de::DeserializeOwned};
use serde_serializers::deserialize_u64_from_str_or_int;

use crate::SwapperError;

const SUCCESS_CODE: u64 = 100;
const QUOTE_FAIL_CODE: u64 = 412;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgersResponse {
    #[serde(deserialize_with = "deserialize_u64_from_str_or_int")]
    pub res_code: u64,
    pub res_msg: String,
    pub data: serde_json::Value,
}

impl BridgersResponse {
    pub fn data<T: DeserializeOwned>(self) -> Result<T, SwapperError> {
        match self.res_code {
            SUCCESS_CODE => Ok(serde_json::from_value(self.data)?),
            QUOTE_FAIL_CODE => Err(SwapperError::NoQuoteAvailable),
            _ => Err(SwapperError::ComputeQuoteError(self.res_msg)),
        }
    }
}
