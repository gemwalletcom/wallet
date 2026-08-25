use serde::{Deserialize, Serialize};

use crate::GemstoneError;
use crate::models::transaction::GemTransactionInputType;
use primitives::Account;

pub type GemAccount = Account;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmDestination {
    pub address: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemConfirmInput {
    pub input_type: GemTransactionInputType,
    pub from: GemAccount,
    pub destination: Option<GemConfirmDestination>,
    pub value: String,
    pub memo: Option<String>,
    pub references: Vec<String>,
    pub use_max: bool,
    pub minimum_value: Option<String>,
}

#[uniffi::export]
pub fn confirm_input_encode(input: &GemConfirmInput) -> Result<String, GemstoneError> {
    serde_json::to_string(input).map_err(GemstoneError::from)
}

#[uniffi::export]
pub fn confirm_input_decode(input: &str) -> Result<GemConfirmInput, GemstoneError> {
    serde_json::from_str(input).map_err(GemstoneError::from)
}
