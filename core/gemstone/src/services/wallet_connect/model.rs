use crate::services::transfer::GemTransferData;
use primitives::{Account, Chain, SimulationResult, Wallet, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

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
    Failed { message: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectOutcome {
    pub response: Option<GemWalletConnectResponse>,
    pub failure: Option<GemWalletConnectFailure>,
}

impl GemWalletConnectOutcome {
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
