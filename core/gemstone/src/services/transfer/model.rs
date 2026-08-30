use serde::{Deserialize, Serialize};

use crate::models::custom_types::GemBigInt;
use crate::models::transaction::{GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadMetadata};
use primitives::{SimulationResult, TransactionType, TransferDataOutputAction, TransferDataOutputType};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, uniffi::Record)]
pub struct GemRecipient {
    pub address: String,
    #[uniffi(default = None)]
    pub name: Option<String>,
    #[uniffi(default = None)]
    pub memo: Option<String>,
    #[uniffi(default = [])]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemTransferData {
    pub input_type: GemTransactionInputType,
    pub recipient: GemRecipient,
    pub value: String,
    pub use_max_amount: bool,
    pub minimum_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransferBalance {
    pub available: GemBigInt,
    pub frozen: GemBigInt,
    pub locked: GemBigInt,
    pub withdrawable: GemBigInt,
    pub votes: u32,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransferOutput {
    pub output_type: TransferDataOutputType,
    pub output_action: TransferDataOutputAction,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemPendingTransactionInput {
    pub sender: String,
    pub transfer: GemTransferData,
    pub value: GemBigInt,
    pub transaction_type: TransactionType,
    pub hash: String,
    pub fee: GemTransactionLoadFee,
    pub network_fee: GemBigInt,
    pub metadata: GemTransactionLoadMetadata,
    pub simulation: Option<SimulationResult>,
    pub transaction_index: u32,
    pub transaction_count: u32,
}
