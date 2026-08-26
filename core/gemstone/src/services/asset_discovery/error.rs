use crate::api::GemApiError;
use crate::services::assets::GemAssetError;
use crate::services::subscription::GemSubscriptionError;

#[derive(Debug, uniffi::Error)]
pub enum GemAssetDiscoveryError {
    Api { msg: String },
    Assets { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemAssetDiscoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Assets { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemAssetDiscoveryError {}

impl From<GemApiError> for GemAssetDiscoveryError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemAssetError> for GemAssetDiscoveryError {
    fn from(error: GemAssetError) -> Self {
        Self::Assets { msg: error.to_string() }
    }
}

impl From<GemSubscriptionError> for GemAssetDiscoveryError {
    fn from(error: GemSubscriptionError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
