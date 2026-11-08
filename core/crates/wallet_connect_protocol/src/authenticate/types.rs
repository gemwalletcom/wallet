use serde::{Deserialize, Serialize};

use crate::sign::types::Participant;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateParams {
    pub requester: Participant,
    pub auth_payload: AuthPayload,
    pub expiry_timestamp: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthPayload {
    pub domain: String,
    pub aud: String,
    pub version: String,
    pub nonce: String,
    pub chains: Vec<String>,
    pub iat: String,
    pub nbf: Option<String>,
    pub exp: Option<String>,
    pub statement: Option<String>,
    pub request_id: Option<String>,
    pub resources: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Cacao {
    pub h: CacaoHeader,
    pub p: CacaoPayload,
    pub s: CacaoSignature,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CacaoHeader {
    pub t: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CacaoPayload {
    pub iss: String,
    pub domain: String,
    pub aud: String,
    pub version: String,
    pub nonce: String,
    pub iat: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CacaoSignature {
    pub t: &'static str,
    pub s: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuthenticateResponse {
    pub responder: Participant,
    pub cacaos: Vec<Cacao>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Issuer {
    pub chain_id: String,
    pub address: String,
}

impl Issuer {
    const PREFIX: &'static str = "did:pkh:";

    pub fn new(chain_id: &str, address: &str) -> Self {
        Self {
            chain_id: chain_id.to_string(),
            address: address.to_string(),
        }
    }

    pub fn did(&self) -> String {
        format!("{}{}:{}", Self::PREFIX, self.chain_id, self.address)
    }

    pub fn reference(&self) -> &str {
        self.chain_id.split_once(':').map(|(_, reference)| reference).unwrap_or_default()
    }
}
