use crate::{ApplicationMetadata, Chain, DateTime, Wallet, WalletConnectRequest, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionState};

impl WalletConnectRequest {
    pub fn mock(method: &str, params: &str, chain_id: Option<&str>) -> Self {
        Self {
            topic: "test-topic".to_string(),
            method: method.to_string(),
            params: params.to_string(),
            chain_id: chain_id.map(ToString::to_string),
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
            created_at: DateTime::from_timestamp(1_700_000_000),
            expire_at: DateTime::from_timestamp(1_700_604_800),
            metadata: ApplicationMetadata::mock(),
        }
    }
}

impl WalletConnectionSessionProposal {
    pub fn mock_with_wallets(wallets: Vec<Wallet>) -> Self {
        Self {
            default_wallet: Wallet::mock(),
            wallets,
            metadata: ApplicationMetadata::mock(),
        }
    }
}
