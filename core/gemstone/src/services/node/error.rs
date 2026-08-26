#[derive(Debug, uniffi::Error)]
pub enum GemNodeError {
    Store { msg: String },
}

impl std::fmt::Display for GemNodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemNodeError {}
