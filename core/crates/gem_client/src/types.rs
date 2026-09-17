use primitives::ResponseError;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

use crate::{CONTENT_TYPE, ContentType, MULTIPART_FORM_DATA};

#[derive(Debug, Clone)]
pub struct Response {
    pub status: Option<u16>,
    pub data: Vec<u8>,
}

#[derive(Clone, PartialEq)]
pub enum ClientError<E = Vec<u8>> {
    Network(String),
    Timeout,
    Http { status: u16, body: E },
    Response { status: u16, message: String },
    Serialization(String),
}

impl ClientError {
    pub fn decode_body<E: DeserializeOwned>(self) -> ClientError<Option<E>> {
        match self {
            Self::Http { status, body } => ClientError::Http {
                status,
                body: serde_json::from_slice(&body).ok(),
            },
            Self::Network(message) => ClientError::Network(message),
            Self::Timeout => ClientError::Timeout,
            Self::Response { status, message } => ClientError::Response { status, message },
            Self::Serialization(message) => ClientError::Serialization(message),
        }
    }
}

impl fmt::Debug for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => f.debug_tuple("Network").field(msg).finish(),
            Self::Timeout => write!(f, "Timeout"),
            Self::Http { status, body } => {
                let body_str = String::from_utf8_lossy(&body[..body.len().min(256)]);
                f.debug_struct("Http").field("status", status).field("body", &body_str).finish()
            }
            Self::Response { status, message } => f.debug_struct("Response").field("status", status).field("message", message).finish(),
            Self::Serialization(msg) => f.debug_tuple("Serialization").field(msg).finish(),
        }
    }
}

pub fn decode_json_byte_array(values: Vec<Value>) -> Result<Vec<u8>, ClientError> {
    let mut bytes = Vec::with_capacity(values.len());
    for value in values {
        let byte = value
            .as_u64()
            .ok_or_else(|| ClientError::Serialization("Expected byte array for binary content-type".to_string()))?;
        if byte > u8::MAX as u64 {
            return Err(ClientError::Serialization("Binary body byte out of range".to_string()));
        }
        bytes.push(byte as u8);
    }
    Ok(bytes)
}

impl<E> fmt::Display for ClientError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::Timeout => write!(f, "Timeout error"),
            Self::Http { status, .. } => write!(f, "HTTP error: status {}", status),
            Self::Response { message, .. } => write!(f, "{}", message),
            Self::Serialization(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ClientError {}

impl From<serde_json::Error> for ClientError {
    fn from(err: serde_json::Error) -> Self {
        ClientError::Serialization(format!("JSON error: {err}"))
    }
}

pub fn encode_request_body<T: Serialize>(headers: &HashMap<String, String>, body: &T) -> Result<Vec<u8>, ClientError> {
    let content_type = headers.get(CONTENT_TYPE).map(String::as_str);
    let is_multipart = content_type.is_some_and(|value| value.starts_with(MULTIPART_FORM_DATA));
    match content_type.and_then(|value| ContentType::from_str(value).ok()) {
        Some(ContentType::TextPlain) | Some(ContentType::ApplicationFormUrlEncoded) => match serde_json::to_value(body)? {
            Value::String(text) => Ok(text.into_bytes()),
            _ => Err(ClientError::Serialization("Expected string body for text content-type".to_string())),
        },
        Some(ContentType::ApplicationXBinary) | Some(ContentType::ApplicationAptosBcs) => decode_binary_body(serde_json::to_value(body)?),
        _ if is_multipart => decode_binary_body(serde_json::to_value(body)?),
        _ => Ok(serde_json::to_vec(body)?),
    }
}

fn decode_binary_body(value: Value) -> Result<Vec<u8>, ClientError> {
    match value {
        Value::String(text) => hex::decode(&text).map_err(|error| ClientError::Serialization(format!("Failed to decode hex string: {error}"))),
        Value::Array(values) => decode_json_byte_array(values),
        _ => Err(ClientError::Serialization("Expected hex string or byte array body for binary content-type".to_string())),
    }
}

pub fn deserialize_response<R>(response: &Response) -> Result<R, ClientError>
where
    R: DeserializeOwned,
{
    let data: &[u8] = if response.data.is_empty() { b"null" } else { &response.data };
    match serde_json::from_slice(data) {
        Ok(value) => Ok(value),
        Err(error) => {
            validate_response(response)?;
            Err(ClientError::Serialization(error.to_string()))
        }
    }
}

pub fn validate_response(response: &Response) -> Result<(), ClientError> {
    if let Ok(body) = serde_json::from_slice::<ResponseError>(&response.data) {
        return Err(ClientError::Response {
            status: response.status.unwrap_or_default(),
            message: body.error.message,
        });
    }
    validate_http_status(response)
}

fn validate_http_status(response: &Response) -> Result<(), ClientError> {
    if let Some(status) = response.status {
        if !(200..400).contains(&status) {
            return Err(ClientError::Http {
                status,
                body: response.data.clone(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const USERNAME_ERROR: &str = r#"{"error":{"message":"Username must contain only letters and digits"}}"#;
    const USERNAME_ERROR_MESSAGE: &str = "Username must contain only letters and digits";

    #[test]
    fn test_validate_response() {
        assert_eq!(
            validate_response(&Response {
                status: Some(200),
                data: USERNAME_ERROR.as_bytes().to_vec()
            }),
            Err(ClientError::Response {
                status: 200,
                message: USERNAME_ERROR_MESSAGE.to_string()
            })
        );
        assert_eq!(
            validate_response(&Response {
                status: Some(400),
                data: USERNAME_ERROR.as_bytes().to_vec()
            }),
            Err(ClientError::Response {
                status: 400,
                message: USERNAME_ERROR_MESSAGE.to_string()
            })
        );
        assert_eq!(
            validate_response(&Response {
                status: Some(502),
                data: b"Bad Gateway".to_vec()
            }),
            Err(ClientError::Http {
                status: 502,
                body: b"Bad Gateway".to_vec()
            })
        );
        assert_eq!(
            validate_response(&Response {
                status: Some(200),
                data: br#"{"points":1}"#.to_vec()
            }),
            Ok(())
        );
    }

    #[test]
    fn test_deserialize_response() {
        assert_eq!(
            deserialize_response::<bool>(&Response {
                status: Some(200),
                data: b"true".to_vec()
            }),
            Ok(true)
        );
        assert_eq!(
            deserialize_response::<bool>(&Response {
                status: Some(200),
                data: USERNAME_ERROR.as_bytes().to_vec()
            }),
            Err(ClientError::Response {
                status: 200,
                message: USERNAME_ERROR_MESSAGE.to_string()
            })
        );
    }
}
