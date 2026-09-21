use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestError {
    Forbidden,
    LimitReached,
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Forbidden => write!(f, "Forbidden"),
            Self::LimitReached => write!(f, "Rate limit reached"),
        }
    }
}

impl std::error::Error for RequestError {}

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
        ResponseResult::Error(ResponseError { error: ErrorDetail { message, data: None } })
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
