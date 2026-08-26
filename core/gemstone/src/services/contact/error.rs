use crate::services::name::GemNameError;

#[derive(Debug, uniffi::Error)]
pub enum GemContactError {
    Store { msg: String },
}

impl std::fmt::Display for GemContactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemContactError {}

impl From<GemNameError> for GemContactError {
    fn from(error: GemNameError) -> Self {
        Self::Store { msg: error.to_string() }
    }
}
