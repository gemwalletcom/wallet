use crate::{ApplicationMetadata, Chain, DateTime, Wallet, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionState};

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
