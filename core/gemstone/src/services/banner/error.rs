#[derive(Debug, uniffi::Error)]
pub enum GemBannerError {
    Store { msg: String },
}

impl std::fmt::Display for GemBannerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store { msg } => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for GemBannerError {}
