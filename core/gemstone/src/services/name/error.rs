use crate::api::GemApiError;

#[derive(Debug, uniffi::Error)]
pub enum GemNameError {
    Api { msg: String },
    Store { msg: String },
}

impl std::fmt::Display for GemNameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api { msg } | Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemNameError {}

impl From<GemApiError> for GemNameError {
    fn from(error: GemApiError) -> Self {
        Self::Api { msg: error.to_string() }
    }
}
