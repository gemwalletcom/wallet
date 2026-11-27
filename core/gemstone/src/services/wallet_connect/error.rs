use crate::services::error::GemServiceError;
use crate::services::wallet_connect::model::GemWalletConnectRejectionReason;
use wallet_connect_protocol::sign::namespaces::NamespaceError;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum GemWalletConnectError {
    UnsupportedChains,
    InvalidOrigin,
    UnsupportedWallets,
    UnsupportedMethods,
    UnsupportedEvents,
    Service { msg: String },
}

#[uniffi::export]
impl GemWalletConnectError {
    pub fn rejection_reason(&self) -> GemWalletConnectRejectionReason {
        match self {
            Self::UnsupportedChains => GemWalletConnectRejectionReason::UnsupportedChains,
            Self::UnsupportedWallets => GemWalletConnectRejectionReason::UnsupportedAccounts,
            Self::UnsupportedMethods => GemWalletConnectRejectionReason::UnsupportedMethods,
            Self::UnsupportedEvents => GemWalletConnectRejectionReason::UnsupportedEvents,
            Self::InvalidOrigin | Self::Service { .. } => GemWalletConnectRejectionReason::Refused,
        }
    }
}

impl std::fmt::Display for GemWalletConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedChains => write!(f, "unsupported chains"),
            Self::InvalidOrigin => write!(f, "invalid origin"),
            Self::UnsupportedWallets => write!(f, "wallets unsupported"),
            Self::UnsupportedMethods => write!(f, "unsupported methods"),
            Self::UnsupportedEvents => write!(f, "unsupported events"),
            Self::Service { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemWalletConnectError {}

impl From<NamespaceError> for GemWalletConnectError {
    fn from(error: NamespaceError) -> Self {
        match error {
            NamespaceError::UnsupportedChains => Self::UnsupportedChains,
            NamespaceError::UnsupportedAccounts => Self::UnsupportedWallets,
            NamespaceError::UnsupportedMethods => Self::UnsupportedMethods,
            NamespaceError::UnsupportedEvents => Self::UnsupportedEvents,
        }
    }
}

impl From<GemServiceError> for GemWalletConnectError {
    fn from(error: GemServiceError) -> Self {
        Self::Service { msg: error.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wallet_connect_protocol::sign::Reason;

    #[test]
    fn test_a_failed_proposal_is_rejected_for_the_reason_that_failed_it() {
        let code = |error: GemWalletConnectError| Reason::from(error.rejection_reason()).code;

        assert_eq!(code(GemWalletConnectError::UnsupportedChains), 5100);
        assert_eq!(code(GemWalletConnectError::UnsupportedWallets), 5103);
        assert_eq!(code(GemWalletConnectError::InvalidOrigin), 5000, "a dApp is not told its chains were unsupported when the origin was the problem");
        assert_eq!(code(GemWalletConnectError::Service { msg: "offline".to_string() }), 5000);
    }
}
