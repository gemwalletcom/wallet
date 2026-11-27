use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::sign::namespaces::{NamespaceError, proposal_chains};

pub const RELAY_PROTOCOL: &str = "irn";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub url: String,
    #[serde(default)]
    pub icons: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect: Option<Redirect>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Redirect {
    pub native: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub universal: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Participant {
    pub public_key: String,
    pub metadata: Metadata,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relay {
    pub protocol: String,
}

impl Relay {
    pub fn irn() -> Self {
        Self { protocol: RELAY_PROTOCOL.to_string() }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProposalNamespace {
    pub chains: Option<Vec<String>>,
    #[serde(default)]
    pub methods: Vec<String>,
    #[serde(default)]
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionNamespace {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chains: Option<Vec<String>>,
    pub accounts: Vec<String>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionProposal {
    pub proposer: Participant,
    #[serde(default)]
    pub required_namespaces: BTreeMap<String, ProposalNamespace>,
    pub optional_namespaces: Option<BTreeMap<String, ProposalNamespace>>,
    pub session_properties: Option<HashMap<String, String>>,
    pub expiry_timestamp: Option<u64>,
}

impl SessionProposal {
    pub fn required_chain_ids(&self) -> Vec<String> {
        chain_ids(&self.required_namespaces)
    }

    pub fn optional_chain_ids(&self) -> Vec<String> {
        self.optional_namespaces.as_ref().map(chain_ids).unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposeResponse {
    pub relay: Relay,
    pub responder_public_key: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettleParams {
    pub relay: Relay,
    pub controller: Participant,
    pub namespaces: BTreeMap<String, SessionNamespace>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub session_properties: HashMap<String, String>,
    pub expiry: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    pub request: RequestBody,
    pub chain_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBody {
    pub method: String,
    pub params: Box<RawValue>,
    pub expiry_timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UpdateParams<'a> {
    pub namespaces: &'a BTreeMap<String, SessionNamespace>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Reason {
    pub code: i32,
    pub message: String,
}

impl Reason {
    const UNAUTHORIZED_METHOD: i32 = 3001;
    const UNAUTHORIZED_CHAIN: i32 = 3005;
    const USER_REJECTED_REQUEST: i32 = 4001;
    const UNRECOGNIZED_CHAIN: i32 = 4902;
    const USER_REJECTED_SESSION: i32 = 5000;
    const UNSUPPORTED_CHAINS: i32 = 5100;
    const UNSUPPORTED_METHODS: i32 = 5101;
    const UNSUPPORTED_EVENTS: i32 = 5102;
    const UNSUPPORTED_ACCOUNTS: i32 = 5103;
    const USER_DISCONNECTED: i32 = 6000;
    const REQUEST_EXPIRED: i32 = 8000;
    const METHOD_NOT_FOUND: i32 = -32601;

    fn new(code: i32, message: impl Into<String>) -> Self {
        Self { code, message: message.into() }
    }

    pub fn unauthorized_chain() -> Self {
        Self::new(Self::UNAUTHORIZED_CHAIN, "Unauthorized target chain id requested")
    }

    pub fn unauthorized_method(method: &str) -> Self {
        Self::new(Self::UNAUTHORIZED_METHOD, format!("Unauthorized JSON-RPC method requested: {method}"))
    }

    pub fn user_rejected_request() -> Self {
        Self::new(Self::USER_REJECTED_REQUEST, "User rejected the request")
    }

    pub fn unrecognized_chain() -> Self {
        Self::new(Self::UNRECOGNIZED_CHAIN, "Unrecognized chain ID")
    }

    pub fn user_rejected_session() -> Self {
        Self::new(Self::USER_REJECTED_SESSION, "User rejected.")
    }

    pub fn user_disconnected() -> Self {
        Self::new(Self::USER_DISCONNECTED, "User disconnected.")
    }

    pub fn request_expired() -> Self {
        Self::new(Self::REQUEST_EXPIRED, "Session request expired")
    }

    pub fn method_not_found() -> Self {
        Self::new(Self::METHOD_NOT_FOUND, "Method not found")
    }
}

impl From<NamespaceError> for Reason {
    fn from(error: NamespaceError) -> Self {
        match error {
            NamespaceError::UnsupportedChains => Self::new(Self::UNSUPPORTED_CHAINS, "Unsupported chains"),
            NamespaceError::UnsupportedMethods => Self::new(Self::UNSUPPORTED_METHODS, "Unsupported methods"),
            NamespaceError::UnsupportedEvents => Self::new(Self::UNSUPPORTED_EVENTS, "Unsupported events"),
            NamespaceError::UnsupportedAccounts => Self::new(Self::UNSUPPORTED_ACCOUNTS, "Unsupported accounts"),
        }
    }
}

fn chain_ids(namespaces: &BTreeMap<String, ProposalNamespace>) -> Vec<String> {
    namespaces.iter().flat_map(|(key, namespace)| proposal_chains(key, namespace).1).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reason_wire_values() {
        let wire = |reason: Reason| serde_json::to_value(reason).unwrap();

        assert_eq!(wire(Reason::unauthorized_chain()), serde_json::json!({"code": 3005, "message": "Unauthorized target chain id requested"}));
        assert_eq!(wire(Reason::unauthorized_method("eth_sign")), serde_json::json!({"code": 3001, "message": "Unauthorized JSON-RPC method requested: eth_sign"}));
        assert_eq!(wire(Reason::user_rejected_request()), serde_json::json!({"code": 4001, "message": "User rejected the request"}));
        assert_eq!(wire(Reason::unrecognized_chain()), serde_json::json!({"code": 4902, "message": "Unrecognized chain ID"}));
        assert_eq!(wire(Reason::user_rejected_session()), serde_json::json!({"code": 5000, "message": "User rejected."}));
        assert_eq!(wire(NamespaceError::UnsupportedChains.into()), serde_json::json!({"code": 5100, "message": "Unsupported chains"}));
        assert_eq!(wire(NamespaceError::UnsupportedMethods.into()), serde_json::json!({"code": 5101, "message": "Unsupported methods"}));
        assert_eq!(wire(NamespaceError::UnsupportedEvents.into()), serde_json::json!({"code": 5102, "message": "Unsupported events"}));
        assert_eq!(wire(NamespaceError::UnsupportedAccounts.into()), serde_json::json!({"code": 5103, "message": "Unsupported accounts"}));
        assert_eq!(wire(Reason::user_disconnected()), serde_json::json!({"code": 6000, "message": "User disconnected."}));
        assert_eq!(wire(Reason::request_expired()), serde_json::json!({"code": 8000, "message": "Session request expired"}));
        assert_eq!(wire(Reason::method_not_found()), serde_json::json!({"code": -32601, "message": "Method not found"}));
    }
}
