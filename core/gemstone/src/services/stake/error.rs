use crate::api::GemApiError;
use crate::gateway::GatewayError;

#[derive(Debug, uniffi::Error)]
pub enum GemStakeError {
    Gateway { msg: String },
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemStakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gateway { msg } | Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemStakeError {}

impl From<GatewayError> for GemStakeError {
    fn from(error: GatewayError) -> Self {
        Self::Gateway { msg: error.to_string() }
    }
}

impl From<GemApiError> for GemStakeError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}
