pub mod model;
mod recent;
pub mod rules;
mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::services::amount::model::GemAmountError;

use crate::GemstoneError;
use crate::models::transaction::GemTransactionInputType;
use crate::services::confirm::GemConfirmInput;
use primitives::TransactionType;
use primitives::swap::ApprovalData;

pub(crate) use model::GemPendingTransactionInput;
pub use model::{GemConfirmDestination, GemRecentActivity, GemRecipient, GemTransferBalance, GemTransferData, GemTransferOutput};
pub use recent::GemRecentActivityService;
pub use store::GemRecentActivityStore;

#[derive(Default, uniffi::Object)]
pub struct GemTransferService;

#[uniffi::export]
impl GemTransferService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn encode_confirm_input(&self, input: &GemConfirmInput) -> Result<String, GemstoneError> {
        serde_json::to_string(input).map_err(GemstoneError::from)
    }

    pub fn decode_confirm_input(&self, input: String) -> Result<GemConfirmInput, GemstoneError> {
        serde_json::from_str(&input).map_err(GemstoneError::from)
    }

    pub fn approval(&self, input_type: &GemTransactionInputType, transaction_type: TransactionType) -> Result<Option<ApprovalData>, GemstoneError> {
        input_type.approval(transaction_type).map_err(|msg| GemstoneError::AnyError { msg })
    }

    pub fn metadata(&self, input_type: &GemTransactionInputType) -> Result<Option<String>, GemstoneError> {
        let metadata = input_type.metadata().map_err(GemstoneError::from)?;
        metadata.map(|value| serde_json::to_string(&value).map_err(GemstoneError::from)).transpose()
    }

    pub fn available_value(&self, transfer: &GemTransferData, balance: GemTransferBalance) -> Result<String, GemAmountError> {
        Ok(transfer.available_value(&balance)?.to_string())
    }
}
