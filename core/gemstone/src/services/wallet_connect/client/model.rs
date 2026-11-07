use primitives::{Account, ApplicationMetadata, WalletConnectionVerificationStatus};

use crate::services::wallet_connect::model::GemWalletConnectSessionRequest;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectEvent {
    Proposal { proposal: GemWalletConnectProposal },
    Authentication { authentication: GemWalletConnectAuthentication },
    Request { request: GemWalletConnectSessionRequest },
    RequestExpired { topic: String, request_id: String },
    SessionDeleted { topic: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectProposal {
    pub id: String,
    pub pairing_topic: String,
    pub metadata: ApplicationMetadata,
    pub required_chain_ids: Vec<String>,
    pub optional_chain_ids: Vec<String>,
    pub origin: Option<String>,
    pub validation: WalletConnectionVerificationStatus,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectAuthentication {
    pub id: String,
    pub metadata: ApplicationMetadata,
    pub chain_ids: Vec<String>,
    pub domain: String,
    pub origin: Option<String>,
    pub validation: WalletConnectionVerificationStatus,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectRelay {
    pub url: String,
    pub origin: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectAuthMessage {
    pub account: Account,
    pub message: String,
}
