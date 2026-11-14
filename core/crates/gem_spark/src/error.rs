use std::fmt;

#[cfg(feature = "wallet")]
use spark_wallet::SparkWalletError;

#[derive(Debug, PartialEq)]
pub enum LightningError {
    InvalidInvoice(String),
    InvalidTransfer,
    Locked,
    Unauthorized,
    AccountMismatch,
    InvoiceExpired,
    InvoiceNetworkMismatch,
    AmountRequired,
    AmountMismatch { invoice: u64, requested: u64 },
    Wallet(String),
}

impl fmt::Display for LightningError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInvoice(message) => write!(f, "invalid invoice: {message}"),
            Self::InvalidTransfer => write!(f, "invalid lightning transfer"),
            Self::Locked => write!(f, "lightning wallet is locked"),
            Self::Unauthorized => write!(f, "lightning payment is not authorized by this wallet"),
            Self::AccountMismatch => write!(f, "lightning wallet does not match the account address"),
            Self::InvoiceExpired => write!(f, "invoice expired"),
            Self::InvoiceNetworkMismatch => write!(f, "invoice is for another network"),
            Self::AmountRequired => write!(f, "invoice has no amount, an amount is required"),
            Self::AmountMismatch { invoice, requested } => write!(f, "invoice amount {invoice} does not match requested amount {requested}"),
            Self::Wallet(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LightningError {}

#[cfg(feature = "wallet")]
impl From<SparkWalletError> for LightningError {
    fn from(error: SparkWalletError) -> Self {
        Self::Wallet(error.to_string())
    }
}
