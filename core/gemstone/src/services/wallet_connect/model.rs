use primitives::{Account, Chain, SimulationResult, Wallet, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

use crate::message::sign_type::SignMessage;
use crate::wallet_connect::{WalletConnectResponseType, WalletConnectTransaction};

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemWalletConnectRequest {
    pub topic: String,
    pub method: String,
    pub params: String,
    pub chain_id: String,
    pub domain: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectResponse {
    Response { value: WalletConnectResponseType },
    Null,
    MethodNotFound,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemSessionWallets {
    pub default_wallet: Wallet,
    pub wallets: Vec<Wallet>,
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
pub struct GemWalletConnectSignRequest {
    pub session_id: String,
    pub chain: Chain,
    pub wallet: Wallet,
    pub account: Account,
    pub session: WalletConnectionSession,
    pub simulation: SimulationResult,
    pub payload: GemWalletConnectSignPayload,
}

#[derive(Debug, Clone, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemWalletConnectSignPayload {
    Message {
        message: SignMessage,
    },
    Transaction {
        transaction: WalletConnectTransaction,
        action: GemWalletConnectTransactionAction,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectTransactionAction {
    Sign,
    Send,
}
