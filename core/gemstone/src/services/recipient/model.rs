#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipientValidation {
    pub is_valid: bool,
    pub address: String,
    pub shows_error: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Error)]
pub enum GemRecipientError {
    InvalidAddress,
    NameRecordMismatch,
}

impl std::fmt::Display for GemRecipientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAddress => write!(f, "invalid recipient address"),
            Self::NameRecordMismatch => write!(f, "name record does not match the input"),
        }
    }
}

impl std::error::Error for GemRecipientError {}
