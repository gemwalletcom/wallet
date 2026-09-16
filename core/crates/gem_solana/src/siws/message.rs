#[derive(Debug, Clone, PartialEq)]
pub struct SiwsMessage {
    pub domain: String,
    pub address: String,
    pub statement: Option<String>,
    pub uri: Option<String>,
    pub version: Option<String>,
    pub chain_id: Option<String>,
    pub nonce: Option<String>,
    pub issued_at: Option<String>,
    pub expiration_time: Option<String>,
    pub not_before: Option<String>,
    pub request_id: Option<String>,
    pub resources: Vec<String>,
}
