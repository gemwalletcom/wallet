use crate::services::error::GemServiceError;

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
pub enum GemWalletImportError {
    InvalidSecretPhrase,
    InvalidSecretPhraseWords { words: Vec<String> },
    InvalidPrivateKey,
    InvalidAddress,
}

impl std::fmt::Display for GemWalletImportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSecretPhrase => write!(f, "invalid secret phrase"),
            Self::InvalidSecretPhraseWords { words } => write!(f, "invalid secret phrase words: {}", words.join(", ")),
            Self::InvalidPrivateKey => write!(f, "invalid private key"),
            Self::InvalidAddress => write!(f, "invalid address"),
        }
    }
}

impl std::error::Error for GemWalletImportError {}

impl From<GemWalletImportError> for GemServiceError {
    fn from(error: GemWalletImportError) -> Self {
        Self::Status { msg: error.to_string() }
    }
}
