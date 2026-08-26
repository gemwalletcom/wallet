use crate::api::GemApiError;
use crate::services::banner::GemBannerError;

#[derive(Debug, uniffi::Error)]
pub enum GemWalletConfigurationError {
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemWalletConfigurationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemWalletConfigurationError {}

impl From<GemApiError> for GemWalletConfigurationError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemBannerError> for GemWalletConfigurationError {
    fn from(error: GemBannerError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
