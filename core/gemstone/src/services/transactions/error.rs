use crate::api::GemApiError;
use crate::services::assets::GemAssetError;
use crate::services::name::GemNameError;

#[derive(Debug, uniffi::Error)]
pub enum GemTransactionsError {
    Api { msg: String },
    Assets { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemTransactionsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Assets { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemTransactionsError {}

impl From<GemApiError> for GemTransactionsError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemAssetError> for GemTransactionsError {
    fn from(error: GemAssetError) -> Self {
        Self::Assets { msg: error.to_string() }
    }
}

impl From<GemNameError> for GemTransactionsError {
    fn from(error: GemNameError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
