use crate::api::GemApiError;
use crate::services::preferences::GemPreferencesError;
use crate::services::price::GemPriceError;

#[derive(Debug, uniffi::Error)]
pub enum GemAssetError {
    Api { msg: String },
    Store { msg: String },
    Price { msg: String },
}

impl std::fmt::Display for GemAssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } | Self::Price { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemAssetError {}

impl From<GemApiError> for GemAssetError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemPriceError> for GemAssetError {
    fn from(error: GemPriceError) -> Self {
        Self::Price { msg: error.to_string() }
    }
}

impl From<GemPreferencesError> for GemAssetError {
    fn from(error: GemPreferencesError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
