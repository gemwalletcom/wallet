use chrono::Utc;

use crate::{ApplicationMetadata, Chain, WalletConnectRequest, WalletConnectionSession, WalletConnectionState};

impl WalletConnectRequest {
    pub fn mock(method: &str, params: &str, chain_id: Option<&str>) -> Self {
        Self {
            topic: "test-topic".to_string(),
            method: method.to_string(),
            params: params.to_string(),
            chain_id: chain_id.map(|v| v.to_string()),
            domain: "example.com".to_string(),
        }
    }
}

impl WalletConnectionSession {
    pub fn mock(id: &str, chains: &[Chain]) -> Self {
        Self {
            id: id.to_string(),
            session_id: id.to_string(),
            state: WalletConnectionState::Active,
            chains: chains.to_vec(),
            created_at: Utc::now(),
            expire_at: Utc::now(),
            metadata: ApplicationMetadata::mock(),
        }
    }
}
