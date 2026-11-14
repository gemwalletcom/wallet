use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

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
    const USER_DISCONNECTED: i32 = 6000;

    pub fn unauthorized_chain() -> Self {
        Self {
            code: Self::UNAUTHORIZED_CHAIN,
            message: "Unauthorized target chain id requested".to_string(),
        }
    }

    pub fn unauthorized_method(method: &str) -> Self {
        Self {
            code: Self::UNAUTHORIZED_METHOD,
            message: format!("Unauthorized JSON-RPC method requested: {method}"),
        }
    }

    pub fn user_disconnected() -> Self {
        Self {
            code: Self::USER_DISCONNECTED,
            message: "User disconnected.".to_string(),
        }
    }
}

fn chain_ids(namespaces: &BTreeMap<String, ProposalNamespace>) -> Vec<String> {
    namespaces
        .iter()
        .flat_map(|(key, namespace)| match &namespace.chains {
            Some(chains) => chains.clone(),
            None if key.contains(':') => vec![key.clone()],
            None => vec![],
        })
        .collect()
}
