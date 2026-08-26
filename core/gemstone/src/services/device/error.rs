use crate::api::GemApiError;
use crate::services::subscription::GemSubscriptionError;

#[derive(Debug, uniffi::Error)]
pub enum GemDeviceError {
    Api { msg: String },
    Store { msg: String },
    Subscriptions { msg: String },
}

impl std::fmt::Display for GemDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } | Self::Subscriptions { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemDeviceError {}

impl From<GemApiError> for GemDeviceError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}

impl From<GemSubscriptionError> for GemDeviceError {
    fn from(error: GemSubscriptionError) -> Self {
        Self::Subscriptions { msg: error.to_string() }
    }
}
