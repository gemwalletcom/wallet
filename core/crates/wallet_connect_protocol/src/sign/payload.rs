use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::ProtocolError;
use crate::sign::types::Reason;

const JSONRPC_VERSION: &str = "2.0";

#[derive(Debug)]
pub enum Payload {
    Request { id: u64, method: String, params: Box<RawValue> },
    Response { id: u64 },
}

#[derive(Deserialize)]
struct Incoming {
    id: u64,
    method: Option<String>,
    params: Option<Box<RawValue>>,
}

#[derive(Serialize)]
struct Request<'a, P: Serialize> {
    id: u64,
    jsonrpc: &'static str,
    method: &'a str,
    params: P,
}

#[derive(Serialize)]
struct Success<'a, R: Serialize> {
    id: u64,
    jsonrpc: &'static str,
    result: &'a R,
}

#[derive(Serialize)]
struct Failure<'a> {
    id: u64,
    jsonrpc: &'static str,
    error: &'a Reason,
}

#[cfg(test)]
impl PartialEq for Payload {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Request { id, method, params },
                Self::Request {
                    id: other_id,
                    method: other_method,
                    params: other_params,
                },
            ) => id == other_id && method == other_method && params.get() == other_params.get(),
            (Self::Response { id }, Self::Response { id: other_id }) => id == other_id,
            (Self::Request { .. }, Self::Response { .. }) | (Self::Response { .. }, Self::Request { .. }) => false,
        }
    }
}

impl Payload {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let incoming: Incoming = serde_json::from_slice(bytes)?;
        Ok(match incoming.method {
            Some(method) => Self::Request {
                id: incoming.id,
                method,
                params: incoming.params.ok_or_else(|| ProtocolError::InvalidMessage("request without params".to_string()))?,
            },
            None => Self::Response { id: incoming.id },
        })
    }
}

pub fn request<P: Serialize>(id: u64, method: &str, params: P) -> Vec<u8> {
    serde_json::to_vec(&Request {
        id,
        jsonrpc: JSONRPC_VERSION,
        method,
        params,
    })
    .unwrap_or_default()
}

pub fn success<R: Serialize>(id: u64, result: &R) -> Vec<u8> {
    serde_json::to_vec(&Success { id, jsonrpc: JSONRPC_VERSION, result }).unwrap_or_default()
}

pub fn failure(id: u64, error: &Reason) -> Vec<u8> {
    serde_json::to_vec(&Failure { id, jsonrpc: JSONRPC_VERSION, error }).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testkit::{TEST_PLAINTEXT, raw_json};

    #[test]
    fn test_parse() {
        assert_eq!(
            Payload::parse(TEST_PLAINTEXT.as_bytes()).unwrap(),
            Payload::Request {
                id: 1,
                method: "wc_sessionPing".to_string(),
                params: raw_json("{}"),
            }
        );
        assert_eq!(
            Payload::parse(br#"{"id":5,"method":"wc_sessionRequest","params":{"value": 123456789012345678901234567890}}"#).unwrap(),
            Payload::Request {
                id: 5,
                method: "wc_sessionRequest".to_string(),
                params: raw_json(r#"{"value": 123456789012345678901234567890}"#),
            },
            "params reach the request handlers byte for byte, so no amount is rounded"
        );
        assert_eq!(Payload::parse(br#"{"id":2,"jsonrpc":"2.0","result":true}"#).unwrap(), Payload::Response { id: 2 });
        assert_eq!(Payload::parse(br#"{"id":3,"jsonrpc":"2.0","error":{"code":5000,"message":"rejected"}}"#).unwrap(), Payload::Response { id: 3 });
        assert!(Payload::parse(br#"{"id":"text","method":"wc_sessionPing"}"#).is_err());
        assert!(Payload::parse(br#"{"id":4,"method":"wc_sessionPing"}"#).is_err());
    }

    #[test]
    fn test_encode() {
        assert_eq!(
            request(
                1,
                "wc_sessionDelete",
                Reason {
                    code: 6000,
                    message: "User disconnected".to_string()
                }
            ),
            br#"{"id":1,"jsonrpc":"2.0","method":"wc_sessionDelete","params":{"code":6000,"message":"User disconnected"}}"#
        );
        assert_eq!(success(2, &true), br#"{"id":2,"jsonrpc":"2.0","result":true}"#);
        assert_eq!(success(3, &raw_json(r#"{"signature":"0x01"}"#)), br#"{"id":3,"jsonrpc":"2.0","result":{"signature":"0x01"}}"#);
        assert_eq!(
            failure(
                4,
                &Reason {
                    code: 4001,
                    message: "User rejected".to_string()
                }
            ),
            br#"{"id":4,"jsonrpc":"2.0","error":{"code":4001,"message":"User rejected"}}"#
        );
    }
}
