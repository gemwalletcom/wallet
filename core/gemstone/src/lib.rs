pub mod address;
pub mod address_formatter;
pub mod alien;
pub mod api;
mod application;
pub mod auth;
pub mod balance_calculator;
pub mod block_explorer;
pub mod clock;
pub mod config;
pub mod crypto_fiat_converter;
pub mod deeplink;
pub mod device;
pub mod fee;
pub mod gateway;
pub mod gem_swapper;
pub mod keystore;
pub mod message;
pub mod mnemonic;
pub mod models;
pub mod network;
pub mod payment;
pub mod perpetual;
pub mod price;
pub mod price_alert_formatter;
pub mod service_status;
pub mod services;
pub mod signer;
pub mod siwe;
pub mod support;
#[cfg(all(test, feature = "reqwest_provider"))]
pub(crate) mod testkit;
pub mod transaction_simulation;
pub mod transaction_state;
pub mod transfer_amount;
pub mod url_action;
pub mod wallet_connect;

use alien::AlienError;

uniffi::setup_scaffolding!("gemstone");
const LIB_VERSION: &str = env!("CARGO_PKG_VERSION");

#[uniffi::export]
pub fn lib_version() -> String {
    String::from(LIB_VERSION)
}

/// GemstoneError
#[derive(Debug, PartialEq, Eq, uniffi::Error)]
pub enum GemstoneError {
    AnyError { msg: String },
    SignerError { error: signer::GemSignerError, msg: String },
}

impl std::fmt::Display for GemstoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyError { msg } | Self::SignerError { msg, .. } => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for GemstoneError {}

impl From<uniffi::UnexpectedUniFFICallbackError> for GemstoneError {
    fn from(error: uniffi::UnexpectedUniFFICallbackError) -> Self {
        Self::AnyError { msg: error.reason }
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<&str> for GemstoneError {
    fn from(error: &str) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<number_formatter::NumberFormatterError> for GemstoneError {
    fn from(error: number_formatter::NumberFormatterError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<String> for GemstoneError {
    fn from(error: String) -> Self {
        Self::AnyError { msg: error }
    }
}

impl From<Box<dyn std::error::Error>> for GemstoneError {
    fn from(error: Box<dyn std::error::Error>) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<primitives::payment_decoder::PaymentDecoderError> for GemstoneError {
    fn from(error: primitives::payment_decoder::PaymentDecoderError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<AlienError> for GemstoneError {
    fn from(error: AlienError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<primitives::SignerError> for GemstoneError {
    fn from(error: primitives::SignerError) -> Self {
        Self::SignerError { msg: error.to_string(), error }
    }
}

impl From<gem_keystore::KeystoreError> for GemstoneError {
    fn from(error: gem_keystore::KeystoreError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<gem_derivation::AccountDerivationError> for GemstoneError {
    fn from(error: gem_derivation::AccountDerivationError) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<serde_json::Error> for GemstoneError {
    fn from(error: serde_json::Error) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}

impl From<std::string::FromUtf8Error> for GemstoneError {
    fn from(error: std::string::FromUtf8Error) -> Self {
        Self::AnyError { msg: error.to_string() }
    }
}
