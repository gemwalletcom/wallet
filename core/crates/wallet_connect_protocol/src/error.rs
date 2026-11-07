use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtocolError {
    InvalidUri(String),
    ExpiredUri,
    InvalidKey,
    InvalidEnvelope,
    Decryption,
    InvalidMessage(String),
    Random,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUri(reason) => write!(f, "invalid WalletConnect URI: {reason}"),
            Self::ExpiredUri => write!(f, "WalletConnect URI has expired"),
            Self::InvalidKey => write!(f, "invalid WalletConnect key"),
            Self::InvalidEnvelope => write!(f, "invalid WalletConnect envelope"),
            Self::Decryption => write!(f, "WalletConnect message could not be decrypted"),
            Self::InvalidMessage(reason) => write!(f, "invalid WalletConnect message: {reason}"),
            Self::Random => write!(f, "random generation failed"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<serde_json::Error> for ProtocolError {
    fn from(error: serde_json::Error) -> Self {
        Self::InvalidMessage(error.to_string())
    }
}
