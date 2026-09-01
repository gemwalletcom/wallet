use crate::services::transfer::GemTransferData;
use primitives::{Account, Chain, SimulationResult, Wallet, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

use crate::message::sign_type::SignMessage;
use crate::wallet_connect::WalletConnectResponseType;

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: String,
    pub domain: String,
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
    MethodNotFound,
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
