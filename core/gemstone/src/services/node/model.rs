use primitives::Latency;

use crate::gateway::GatewayError;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemNodeStatusState {
    Loading,
    Error,
    Result { latest_block_number: u64, latency: Latency },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemNodeCheck {
    pub url: String,
    pub chain_id: String,
    pub latest_block_number: u64,
    pub is_in_sync: bool,
    pub latency: Latency,
}

#[derive(Debug, Clone, uniffi::Error)]
pub enum GemAddNodeError {
    InvalidUrl,
    InvalidNetworkId,
    Gateway(GatewayError),
}

impl std::fmt::Display for GemAddNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidUrl => write!(f, "invalid node url"),
            Self::InvalidNetworkId => write!(f, "node answered for a different network"),
            Self::Gateway(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for GemAddNodeError {}

impl From<GatewayError> for GemAddNodeError {
    fn from(error: GatewayError) -> Self {
        match error {
            GatewayError::NetworkIdMismatch { .. } => Self::InvalidNetworkId,
            error => Self::Gateway(error),
        }
    }
}
