use serde::{Deserialize, Serialize};

use crate::models::custom_types::GemBigInt;
use crate::models::transaction::{GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadMetadata};
use primitives::{AssetId, RecentActivityType, SimulationResult, TransactionType, TransferDataOutputAction, TransferDataOutputType};

/// What a completed transfer adds to the wallet's recent activity, if anything.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecentActivity {
    pub activity_type: RecentActivityType,
    pub asset_id: AssetId,
    pub to_asset_id: Option<AssetId>,
}

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
    #[serde(with = "crate::models::custom_types::decimal_string")]
    pub value: GemBigInt,
    pub use_max_amount: bool,
    #[serde(with = "crate::models::custom_types::decimal_string::optional")]
    pub minimum_value: Option<GemBigInt>,
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

#[cfg(test)]
mod wire_format_tests {
    use super::*;

    #[test]
    fn test_transfer_value_keeps_the_decimal_string_wire_format() {
        let json = r#"{"address":"recipient","name":null,"memo":null,"references":[]}"#;
        let recipient: GemRecipient = serde_json::from_str(json).unwrap();
        assert_eq!(serde_json::to_string(&recipient).unwrap(), json);
    }

    #[test]
    fn test_a_malformed_transfer_value_is_rejected_rather_than_read_as_zero() {
        let malformed =
            r#"{"input_type":{},"recipient":{"address":"r","name":null,"memo":null,"references":[]},"value":"not-a-number","use_max_amount":false,"minimum_value":null}"#;
        assert!(serde_json::from_str::<GemTransferData>(malformed).is_err());
    }
}
