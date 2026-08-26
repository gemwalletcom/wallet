use crate::api::GemApiError;
use crate::gateway::GatewayError;

#[derive(Debug, uniffi::Error)]
pub enum GemServiceError {
    Api { msg: String },
    Gateway { msg: String },
    Store { msg: String },
    Status { msg: String },
    UnknownCurrency { currency: String },
}

impl std::fmt::Display for GemServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Gateway { msg } | Self::Store { msg } | Self::Status { msg } => write!(f, "{msg}"),
            Self::UnknownCurrency { currency } => write!(f, "unknown currency: {currency}"),
        }
    }
}

impl std::error::Error for GemServiceError {}

impl From<GemApiError> for GemServiceError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GatewayError> for GemServiceError {
    fn from(error: GatewayError) -> Self {
        Self::Gateway { msg: error.to_string() }
    }
}
