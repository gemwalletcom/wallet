use serde::de::IgnoredAny;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::value::RawValue;

use crate::ProtocolError;
use crate::sign::method::Publish;

const JSONRPC_VERSION: &str = "2.0";
const METHOD_SUBSCRIPTION: &str = "irn_subscription";

#[derive(Serialize)]
struct Request<'a, P: Serialize> {
    id: u64,
    jsonrpc: &'static str,
    method: &'a str,
    params: P,
}

#[derive(Serialize)]
struct Acknowledgement<'a> {
    id: &'a Value,
    jsonrpc: &'static str,
    result: bool,
}

#[derive(Serialize)]
struct TopicParams<'a> {
    topic: &'a str,
}

#[derive(Serialize)]
struct TopicsParams<'a> {
    topics: &'a [String],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PublishParams<'a> {
    topic: &'a str,
    message: &'a str,
    ttl: u64,
    tag: u32,
    prompt: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApproveSession<'a> {
    pub pairing_topic: &'a str,
    pub session_topic: &'a str,
    pub session_proposal_response: &'a str,
    pub session_settlement_request: &'a str,
    pub correlation_id: u64,
    pub approved_chains: Vec<String>,
    pub approved_methods: Vec<String>,
    pub approved_events: Vec<String>,
}

#[derive(Deserialize)]
struct Incoming {
    id: Value,
    method: Option<String>,
    params: Option<Box<RawValue>>,
    error: Option<IgnoredAny>,
}

#[derive(Deserialize)]
struct SubscriptionParams {
    data: RelayMessage,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RelayMessage {
    pub topic: String,
    pub message: String,
    pub attestation: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum RelayFrame {
    Message { id: Value, message: RelayMessage },
    Accepted { id: Option<u64> },
    Refused,
}

pub fn subscribe(id: u64, topic: &str) -> String {
    request(id, "irn_subscribe", TopicParams { topic })
}

pub fn batch_subscribe(id: u64, topics: &[String]) -> String {
    request(id, "irn_batchSubscribe", TopicsParams { topics })
}

pub fn publish(id: u64, topic: &str, message: &str, method: Publish) -> String {
    request(
        id,
        "irn_publish",
        PublishParams {
            topic,
            message,
            ttl: method.ttl(),
            tag: method.tag(),
            prompt: false,
        },
    )
}

pub fn approve_session(id: u64, params: ApproveSession) -> String {
    request(id, "wc_approveSession", params)
}

pub fn acknowledge(id: &Value) -> String {
    serde_json::to_string(&Acknowledgement { id, jsonrpc: JSONRPC_VERSION, result: true }).unwrap_or_default()
}

pub fn parse_frame(frame: &str) -> Result<RelayFrame, ProtocolError> {
    let incoming: Incoming = serde_json::from_str(frame)?;
    match incoming.method.as_deref() {
        Some(METHOD_SUBSCRIPTION) => {
            let params = incoming.params.ok_or_else(|| ProtocolError::InvalidMessage("subscription without params".to_string()))?;
            let params: SubscriptionParams = serde_json::from_str(params.get())?;
            Ok(RelayFrame::Message { id: incoming.id, message: params.data })
        }
        Some(method) => Err(ProtocolError::InvalidMessage(format!("unexpected relay request {method}"))),
        None if incoming.error.is_some() => Ok(RelayFrame::Refused),
        None => Ok(RelayFrame::Accepted { id: incoming.id.as_u64() }),
    }
}

fn request<P: Serialize>(id: u64, method: &str, params: P) -> String {
    serde_json::to_string(&Request {
        id,
        jsonrpc: JSONRPC_VERSION,
        method,
        params,
    })
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frame() {
        assert_eq!(
            parse_frame(include_str!("../../testdata/relay_subscription.json")).unwrap(),
            RelayFrame::Message {
                id: Value::from(1_700_000_000_000_123_456u64),
                message: RelayMessage {
                    topic: "eec4db5e6f3e1b8aab5f72a3a8f293882aad0e966f2d7170ccdbdbe1f4f5bca6".to_string(),
                    message: "AAABAg==".to_string(),
                    attestation: Some("eyJhbGciOiJFUzI1NiJ9.e30.c2ln".to_string()),
                },
            }
        );
        assert_eq!(parse_frame(r#"{"id":1,"jsonrpc":"2.0","result":true}"#).unwrap(), RelayFrame::Accepted { id: Some(1) });
        assert_eq!(parse_frame(r#"{"id":2,"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid"}}"#).unwrap(), RelayFrame::Refused);
        assert_eq!(
            parse_frame(r#"{"id":1,"jsonrpc":"2.0","method":"irn_publish","params":{}}"#),
            Err(ProtocolError::InvalidMessage("unexpected relay request irn_publish".to_string()))
        );
        assert!(parse_frame("{").is_err());
    }

    #[test]
    fn test_requests() {
        assert_eq!(subscribe(1, "topic"), r#"{"id":1,"jsonrpc":"2.0","method":"irn_subscribe","params":{"topic":"topic"}}"#);
        assert_eq!(
            batch_subscribe(2, &["a".to_string(), "b".to_string()]),
            r#"{"id":2,"jsonrpc":"2.0","method":"irn_batchSubscribe","params":{"topics":["a","b"]}}"#
        );
        assert_eq!(
            publish(4, "topic", "message", Publish::SessionRequestResponse),
            r#"{"id":4,"jsonrpc":"2.0","method":"irn_publish","params":{"topic":"topic","message":"message","ttl":300,"tag":1109,"prompt":false}}"#
        );
        assert_eq!(acknowledge(&Value::from(5)), r#"{"id":5,"jsonrpc":"2.0","result":true}"#);
        assert_eq!(
            approve_session(
                6,
                ApproveSession {
                    pairing_topic: "pairing",
                    session_topic: "session",
                    session_proposal_response: "response",
                    session_settlement_request: "settle",
                    correlation_id: 7,
                    approved_chains: vec!["eip155:1".to_string()],
                    approved_methods: vec!["personal_sign".to_string()],
                    approved_events: vec![],
                }
            ),
            r#"{"id":6,"jsonrpc":"2.0","method":"wc_approveSession","params":{"pairingTopic":"pairing","sessionTopic":"session","sessionProposalResponse":"response","sessionSettlementRequest":"settle","correlationId":7,"approvedChains":["eip155:1"],"approvedMethods":["personal_sign"],"approvedEvents":[]}}"#
        );
    }
}
