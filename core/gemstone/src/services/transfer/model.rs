use serde::{Deserialize, Serialize};

use crate::models::custom_types::GemBigInt;
use crate::models::transaction::{GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadMetadata};
use primitives::{AssetId, RecentActivityType, Resource, SimulationResult, TransactionType, TransferDataOutputAction, TransferDataOutputType};

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

impl GemRecipient {
    pub fn address(address: String) -> Self {
        Self {
            address,
            name: None,
            memo: None,
            references: Vec::new(),
        }
    }

    pub fn named(address: String, name: String) -> Self {
        Self {
            name: Some(name),
            ..Self::address(address)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, uniffi::Record)]
pub struct GemTransferData {
    pub input_type: GemTransactionInputType,
    pub recipient: GemRecipient,
    #[serde(with = "crate::models::custom_types::decimal_string")]
    pub value: GemBigInt,
    #[uniffi(default = false)]
    pub use_max_amount: bool,
    #[serde(with = "crate::models::custom_types::decimal_string::optional")]
    #[uniffi(default = None)]
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

pub(crate) struct GemPendingTransactionInput {
    pub(crate) sender: String,
    pub(crate) transfer: GemTransferData,
    pub(crate) value: GemBigInt,
    pub(crate) transaction_type: TransactionType,
    pub(crate) hash: String,
    pub(crate) fee: GemTransactionLoadFee,
    pub(crate) network_fee: GemBigInt,
    pub(crate) metadata: GemTransactionLoadMetadata,
    pub(crate) simulation: Option<SimulationResult>,
    pub(crate) transaction_index: u32,
    pub(crate) transaction_count: u32,
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmDestination {
    Recipient { name: Option<String>, address: String },
    Contract { address: String },
    Validator { name: String, address: String },
    Resource { resource: Resource },
    Provider { name: String, address: String },
}
