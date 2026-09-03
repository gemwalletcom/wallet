pub mod model;
mod recent;
pub mod rules;
mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::GemstoneError;
use crate::models::transaction::GemTransactionInputType;
use crate::services::perpetual::GemPerpetualPositionAction;
use primitives::TransactionType;
use primitives::swap::ApprovalData;

pub(crate) use model::GemPendingTransactionInput;
pub use model::{GemConfirmDestination, GemRecentActivity, GemRecipient, GemTransferData, GemTransferOutput};
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

    pub fn encode_transfer_data(&self, transfer: &GemTransferData) -> Result<String, GemstoneError> {
        serde_json::to_string(transfer).map_err(GemstoneError::from)
    }

    pub fn decode_transfer_data(&self, transfer: String) -> Result<GemTransferData, GemstoneError> {
        serde_json::from_str(&transfer).map_err(GemstoneError::from)
    }

    pub fn encode_position_action(&self, action: &GemPerpetualPositionAction) -> Result<String, GemstoneError> {
        serde_json::to_string(action).map_err(GemstoneError::from)
    }

    pub fn decode_position_action(&self, action: String) -> Result<GemPerpetualPositionAction, GemstoneError> {
        serde_json::from_str(&action).map_err(GemstoneError::from)
    }

    pub fn approval(&self, input_type: &GemTransactionInputType, transaction_type: TransactionType) -> Result<Option<ApprovalData>, GemstoneError> {
        input_type.approval(transaction_type).map_err(|msg| GemstoneError::AnyError { msg })
    }

    pub fn metadata(&self, input_type: &GemTransactionInputType) -> Result<Option<String>, GemstoneError> {
        let metadata = input_type.metadata().map_err(GemstoneError::from)?;
        metadata.map(|value| serde_json::to_string(&value).map_err(GemstoneError::from)).transpose()
    }
}
