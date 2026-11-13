use crate::application::GemConnectionRow;
use crate::formatted_number::GemValueTone;
use crate::models::list::{GemGroup, GemGroupList, GemListRow, GemListRowTitle, GemRowAction};
use crate::services::wallet::model::wallet_row;
use crate::services::wallet_connect::sign_message::network_row;
use crate::services::wallet_session::rules::can_choose_wallet;
use crate::services::error_text::GemErrorText;
use crate::services::localization::GemLocalizedText;
use crate::services::transfer::GemTransferData;
use primitives::{Account, Asset, Chain, SimulationResult, VerificationStatus, Wallet, WalletConnection, WalletId, WalletConnectionSession, WalletConnectionSessionProposal, WalletConnectionVerificationStatus};

use crate::message::sign_type::SignMessage;
use gem_wallet_connect::WalletConnectResponseType;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemWalletConnectRejectionReason {
    UserRejected,
    UnsupportedChains,
    UnsupportedMethods,
    UnsupportedAccounts,
    UnsupportedEvents,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemWalletConnectRejection {
    pub reason: GemWalletConnectRejectionReason,
    pub code: i32,
    pub message: String,
    pub deletes_session: bool,
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
    Failed { error: GemErrorText },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemSignerFailure {
    Retry { error: GemErrorText },
    Reject,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemWalletConnectOutcome {
    Ignored,
    Respond { response: GemWalletConnectResponse },
    Fail { error: GemWalletConnectRpcError, failure: GemWalletConnectFailure },
    AddSessionAccount { account: String },
}

impl GemWalletConnectOutcome {
    pub fn respond_null() -> Self {
        Self::Respond { response: GemWalletConnectResponse::Null }
    }

    pub fn respond_error(error: GemWalletConnectRpcError) -> Self {
        Self::Respond {
            response: GemWalletConnectResponse::Error { error },
        }
    }

    pub fn rejected() -> Self {
        Self::respond_error(crate::services::wallet_connect::rules::user_rejected_error())
    }

    pub fn failed(failure: GemWalletConnectFailure) -> Self {
        Self::Fail {
            error: crate::services::wallet_connect::rules::user_rejected_error(),
            failure,
        }
    }

    pub fn expired() -> Self {
        Self::Fail {
            error: crate::services::wallet_connect::rules::request_expired_error(),
            failure: GemWalletConnectFailure::Expired,
        }
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
    pub request_id: String,
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
    pub request_id: String,
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

fn verification_status(status: WalletConnectionVerificationStatus) -> VerificationStatus {
    match status {
        WalletConnectionVerificationStatus::Verified => VerificationStatus::Verified,
        WalletConnectionVerificationStatus::Unknown => VerificationStatus::Unverified,
        WalletConnectionVerificationStatus::Invalid | WalletConnectionVerificationStatus::Malicious => VerificationStatus::Suspicious,
    }
}

fn verification_row(status: WalletConnectionVerificationStatus) -> GemListRow {
    GemListRow::Verification {
        title: GemListRowTitle::Status,
        status: verification_status(status),
        info: None,
    }
}

fn review_wallet_row(proposal: &WalletConnectionSessionProposal, wallet_id: Option<WalletId>) -> GemListRow {
    let wallet = wallet_id
        .and_then(|wallet_id| proposal.wallets.iter().find(|wallet| wallet.id == wallet_id))
        .unwrap_or(&proposal.default_wallet);
    GemListRow::Wallet {
        title: GemListRowTitle::Wallet,
        wallet: wallet_row(wallet.clone()),
        menu: vec![],
        action: can_choose_wallet(&proposal.wallets).then_some(GemRowAction::SelectWallet),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemConnectionPermission {
    ViewBalance,
    ApprovalRequests,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemConnectionProposal {
    pub sections: Vec<GemGroup>,
    pub permissions: Vec<GemConnectionPermission>,
}

#[uniffi::export]
pub fn connection_proposal(proposal: WalletConnectionSessionProposal, wallet_id: Option<WalletId>, status: WalletConnectionVerificationStatus) -> GemConnectionProposal {
    let connection = GemListRow::Label {
        title: GemListRowTitle::Connection,
        text: GemLocalizedText::RowTitle { title: GemListRowTitle::WalletConnect },
        tone: GemValueTone::Plain,
        info: None,
        progress: false,
    };
    GemConnectionProposal {
        sections: vec![GemGroup::untitled(vec![review_wallet_row(&proposal, wallet_id), connection, verification_row(status)])],
        permissions: vec![GemConnectionPermission::ViewBalance, GemConnectionPermission::ApprovalRequests],
    }
}

#[uniffi::export]
pub fn authentication_sections(proposal: WalletConnectionSessionProposal, wallet_id: Option<WalletId>, chain: Option<Chain>, status: WalletConnectionVerificationStatus) -> Vec<GemGroup> {
    let rows = [Some(review_wallet_row(&proposal, wallet_id)), chain.map(network_row), Some(verification_row(status))];
    vec![GemGroup::untitled(rows.into_iter().flatten().collect())]
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConnection {
    pub connection: WalletConnection,
    pub row: GemConnectionRow,
}

impl PartialEq for GemConnection {
    fn eq(&self, other: &Self) -> bool {
        self.connection.session == other.connection.session && self.row == other.row
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConnectionsView {
    pub list: GemGroupList,
    pub docs_url: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemConnectionDetails {
    pub connection: GemConnection,
    pub rows: Vec<GemListRow>,
}

#[cfg(test)]
mod verification_tests {
    use super::*;
    use primitives::ApplicationMetadata;

    fn proposal(wallets: Vec<Wallet>) -> WalletConnectionSessionProposal {
        WalletConnectionSessionProposal {
            default_wallet: Wallet::mock(),
            wallets,
            metadata: ApplicationMetadata::mock(),
        }
    }

    fn status_of(row: &GemListRow) -> Option<VerificationStatus> {
        match row {
            GemListRow::Verification {
                title: GemListRowTitle::Status,
                status,
                info: None,
            } => Some(*status),
            _ => None,
        }
    }

    fn wallet_action(row: &GemListRow) -> Option<GemRowAction> {
        match row {
            GemListRow::Wallet { action, .. } => action.clone(),
            _ => None,
        }
    }

    #[test]
    fn test_an_invalid_origin_reads_as_suspicious_and_an_unknown_one_only_as_unverified() {
        let status = |status| status_of(&verification_row(status));

        assert_eq!(status(WalletConnectionVerificationStatus::Verified), Some(VerificationStatus::Verified));
        assert_eq!(status(WalletConnectionVerificationStatus::Unknown), Some(VerificationStatus::Unverified));
        assert_eq!(status(WalletConnectionVerificationStatus::Invalid), Some(VerificationStatus::Suspicious));
        assert_eq!(status(WalletConnectionVerificationStatus::Malicious), Some(VerificationStatus::Suspicious));
    }

    #[test]
    fn test_a_proposal_shows_the_wallet_the_connection_and_its_status_and_asks_for_balance_and_approvals() {
        let single = connection_proposal(proposal(vec![Wallet::mock()]), None, WalletConnectionVerificationStatus::Malicious);
        let rows = single.sections[0].rows();

        assert_eq!(single.sections.len(), 1);
        assert_eq!(rows.len(), 3);
        assert_eq!(wallet_action(&rows[0]), None);
        assert_eq!(rows[1].action(), None);
        assert_eq!(status_of(&rows[2]), Some(VerificationStatus::Suspicious));
        assert_eq!(single.permissions, vec![GemConnectionPermission::ViewBalance, GemConnectionPermission::ApprovalRequests]);

        let several = connection_proposal(proposal(vec![Wallet::mock(), Wallet::mock()]), Some(WalletId::mock()), WalletConnectionVerificationStatus::Verified);
        assert_eq!(several.sections[0].rows()[0].action(), Some(GemRowAction::SelectWallet));
    }

    #[test]
    fn test_a_sign_in_shows_the_network_only_once_the_account_is_known() {
        let wallets = vec![Wallet::mock(), Wallet::mock()];
        let with_chain = authentication_sections(proposal(wallets.clone()), Some(WalletId::mock()), Some(Chain::Ethereum), WalletConnectionVerificationStatus::Unknown);
        let without_chain = authentication_sections(proposal(wallets), None, None, WalletConnectionVerificationStatus::Unknown);

        let rows = with_chain[0].rows();
        let without = without_chain[0].rows();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].action(), Some(GemRowAction::SelectWallet));
        assert!(match &rows[1] {
            GemListRow::Network { chain, .. } => *chain == Chain::Ethereum,
            _ => false,
        });
        assert_eq!(status_of(&rows[2]), Some(VerificationStatus::Unverified));
        assert_eq!(without.len(), 2);
        assert_eq!(status_of(&without[1]), Some(VerificationStatus::Unverified));
    }
}
