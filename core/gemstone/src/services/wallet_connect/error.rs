use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum GemWalletConnectError {
    UnsupportedChains,
    InvalidOrigin,
    UnsupportedWallets,
    Service { msg: String },
}

impl std::fmt::Display for GemWalletConnectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedChains => write!(f, "unsupported chains"),
            Self::InvalidOrigin => write!(f, "invalid origin"),
            Self::UnsupportedWallets => write!(f, "wallets unsupported"),
            Self::Service { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemWalletConnectError {}

impl From<GemServiceError> for GemWalletConnectError {
    fn from(error: GemServiceError) -> Self {
        Self::Service { msg: error.to_string() }
    }
}
