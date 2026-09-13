use crate::services::transfer::GemTransferData;
use primitives::{Account, Asset, Chain, SimulationResult, Wallet, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

use crate::message::sign_type::SignMessage;
use crate::wallet_connect::WalletConnectResponseType;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectSessionRequest {
    pub topic: String,
    pub request_id: String,
    pub method: String,
    pub params: String,
    pub chain_id: Option<String>,
    pub origin: Option<String>,
    pub validation: WalletConnectionVerificationStatus,
    pub expiry: Option<u64>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectAuthAccount {
    pub account: Account,
    pub chain_id: String,
    pub issuer: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectRpcError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectResponse {
    Response { value: WalletConnectResponseType },
    Null,
    Error { error: GemWalletConnectRpcError },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectFailure {
    MaliciousOrigin,
    Expired,
    Failed { message: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectOutcome {
    pub response: Option<GemWalletConnectResponse>,
    pub failure: Option<GemWalletConnectFailure>,
}

impl GemWalletConnectOutcome {
    pub fn expired() -> Self {
        Self {
            response: Some(GemWalletConnectResponse::Error {
                error: crate::services::wallet_connect::rules::request_expired_error(),
            }),
            failure: Some(GemWalletConnectFailure::Expired),
        }
    }

    pub fn rejected(failure: Option<GemWalletConnectFailure>) -> Self {
        Self {
            response: Some(GemWalletConnectResponse::Error {
                error: crate::services::wallet_connect::rules::user_rejected_error(),
            }),
            failure,
        }
    }

    pub fn ignored() -> Self {
        Self { response: None, failure: None }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSessionProposal {
    pub proposal: WalletConnectionSessionProposal,
    pub verification_status: WalletConnectionVerificationStatus,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSessionApproval {
    pub chains: Vec<Chain>,
    pub accounts: Vec<Account>,
    pub methods: Vec<String>,
    pub events: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectMessageRequest {
    pub session_id: String,
    pub chain: Chain,
    pub wallet: Wallet,
    pub account: Account,
    pub session: WalletConnectionSession,
    pub simulation: SimulationResult,
    pub message: SignMessage,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectTransactionRequest {
    pub session_id: String,
    pub chain: Chain,
    pub wallet: Wallet,
    pub account: Account,
    pub session: WalletConnectionSession,
    pub simulation: SimulationResult,
    pub transfer: GemTransferData,
    pub action: GemWalletConnectTransactionAction,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectTransactionAction {
    Sign,
    Send,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemVerificationLevel {
    Verified,
    Unverified,
    Suspicious,
}

#[uniffi::export]
pub fn verification_level(status: WalletConnectionVerificationStatus) -> GemVerificationLevel {
    match status {
        WalletConnectionVerificationStatus::Verified => GemVerificationLevel::Verified,
        WalletConnectionVerificationStatus::Unknown => GemVerificationLevel::Unverified,
        WalletConnectionVerificationStatus::Invalid | WalletConnectionVerificationStatus::Malicious => GemVerificationLevel::Suspicious,
    }
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    #[test]
    fn test_an_invalid_origin_reads_as_suspicious_and_an_unknown_one_only_as_unverified() {
        assert_eq!(verification_level(WalletConnectionVerificationStatus::Verified), GemVerificationLevel::Verified);
        assert_eq!(verification_level(WalletConnectionVerificationStatus::Unknown), GemVerificationLevel::Unverified);
        assert_eq!(verification_level(WalletConnectionVerificationStatus::Invalid), GemVerificationLevel::Suspicious);
        assert_eq!(verification_level(WalletConnectionVerificationStatus::Malicious), GemVerificationLevel::Suspicious);
    }
}
