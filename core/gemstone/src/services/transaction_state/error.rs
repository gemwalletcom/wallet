#[derive(Debug, uniffi::Error)]
pub enum GemTransactionStateError {
    Store { msg: String },
    Status { msg: String },
}

impl std::fmt::Display for GemTransactionStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store { msg } | Self::Status { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemTransactionStateError {}
