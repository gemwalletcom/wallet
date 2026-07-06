use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult<T> {
    Success(T),
    Error(ResponseError),
}

impl<T> ResponseResult<T> {
    pub fn new(data: T) -> Self {
        ResponseResult::Success(data)
    }

    pub fn error(message: String) -> Self {
        ResponseResult::Error(ResponseError {
            error: ErrorDetail { message, data: None },
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseError {
    pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}
