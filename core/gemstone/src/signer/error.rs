use primitives::SignerError;

#[uniffi::remote(Enum)]
pub enum SignerError {
    InvalidInput(String),
    SigningError(String),
    DustThreshold,
    InsufficientFunds,
}
