use primitives::SignerError;

pub type GemSignerError = SignerError;

#[uniffi::remote(Enum)]
pub enum GemSignerError {
    InvalidInput(String),
    SigningError(String),
    InsufficientFunds,
    SwapValueBelowMinimum,
}
