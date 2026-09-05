use crate::GemstoneError;
use crate::api::GemApiError;
use crate::gateway::GatewayError;

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemServiceError {
    Api { msg: String },
    Gateway { msg: String },
    Store { msg: String },
    Core { msg: String },
    Platform { msg: String },
    InvalidInput { msg: String },
    NotFound { msg: String },
    Unsupported { msg: String },
    Cancelled,
}

impl std::fmt::Display for GemServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg }
            | Self::Gateway { msg }
            | Self::Store { msg }
            | Self::Core { msg }
            | Self::Platform { msg }
            | Self::InvalidInput { msg }
            | Self::NotFound { msg }
            | Self::Unsupported { msg } => write!(f, "{msg}"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl std::error::Error for GemServiceError {}

impl From<uniffi::UnexpectedUniFFICallbackError> for GemServiceError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::Platform { msg: error.reason }
    }
}

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

impl From<GemstoneError> for GemServiceError {
    fn from(error: GemstoneError) -> Self {
        Self::Core { msg: error.to_string() }
    }
}
