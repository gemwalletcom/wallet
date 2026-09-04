pub mod model;
mod recent;
pub mod rules;
mod store;
#[cfg(test)]
pub(crate) mod testkit;

use crate::GemstoneError;
use crate::services::perpetual::GemPerpetualPositionAction;

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
}
