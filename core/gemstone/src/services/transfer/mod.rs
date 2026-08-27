pub mod model;
pub mod rules;

use crate::GemstoneError;
use crate::models::transaction::GemTransactionInputType;
use primitives::swap::ApprovalData;
use primitives::{Asset, AssetId, Transaction, TransactionType};

pub use model::{GemPendingTransactionInput, GemRecipient, GemTransferBalance, GemTransferData, GemTransferOutput};

#[derive(Default, uniffi::Object)]
pub struct GemTransferService;

#[uniffi::export]
impl GemTransferService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self
    }

    pub fn transaction_type(&self, input_type: &GemTransactionInputType) -> TransactionType {
        rules::transaction_type(input_type)
    }

    pub fn asset(&self, input_type: &GemTransactionInputType) -> Asset {
        rules::asset(input_type)
    }

    pub fn asset_ids(&self, input_type: &GemTransactionInputType) -> Vec<AssetId> {
        rules::asset_ids(input_type)
    }

    pub fn fee_asset(&self, input_type: &GemTransactionInputType) -> Asset {
        rules::fee_asset(input_type)
    }

    pub fn output(&self, input_type: &GemTransactionInputType) -> GemTransferOutput {
        rules::output(input_type)
    }

    pub fn approval(&self, input_type: &GemTransactionInputType, transaction_type: TransactionType) -> Result<Option<ApprovalData>, GemstoneError> {
        rules::approval(input_type, transaction_type).map_err(|msg| GemstoneError::AnyError { msg })
    }

    pub fn metadata(&self, input_type: &GemTransactionInputType) -> Result<Option<String>, GemstoneError> {
        let metadata = rules::metadata(input_type).map_err(GemstoneError::from)?;
        metadata.map(|value| serde_json::to_string(&value).map_err(GemstoneError::from)).transpose()
    }

    pub fn available_value(&self, transfer: &GemTransferData, balance: GemTransferBalance) -> String {
        rules::available_value(transfer, &balance).to_string()
    }

    pub fn pending_transaction(&self, input: GemPendingTransactionInput) -> Result<Option<Transaction>, GemstoneError> {
        rules::pending_transaction(input).map_err(|msg| GemstoneError::AnyError { msg })
    }
}
