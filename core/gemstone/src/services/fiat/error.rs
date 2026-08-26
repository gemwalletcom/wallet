use crate::api::GemApiError;
use crate::services::assets::GemAssetError;

#[derive(Debug, uniffi::Error)]
pub enum GemFiatError {
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemFiatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemFiatError {}

impl From<GemApiError> for GemFiatError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemAssetError> for GemFiatError {
    fn from(error: GemAssetError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
