use crate::models::custom_types::GemBigInt;
use crate::models::transaction::{GemTransactionInputType, GemTransactionLoadFee, GemTransactionLoadMetadata};
use primitives::{AssetId, RecentActivityType, Resource, SimulationResult, TransactionType, TransferDataOutputAction, TransferDataOutputType};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecentActivity {
    pub activity_type: RecentActivityType,
    pub asset_id: AssetId,
    pub to_asset_id: Option<AssetId>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemRecipient {
    pub address: String,
    #[uniffi(default = None)]
    pub name: Option<String>,
    #[uniffi(default = None)]
    pub memo: Option<String>,
    #[uniffi(default = [])]
    pub references: Vec<String>,
}

#[uniffi::export]
impl GemTransferData {
    pub fn identifier(&self) -> String {
        [self.input_type.asset().chain().as_ref(), &self.recipient.address, &self.value.to_string()].join("-")
    }
}

#[uniffi::export]
impl GemRecipient {
    pub fn identifier(&self) -> String {
        [self.name.as_deref().unwrap_or_default(), &self.address, self.memo.as_deref().unwrap_or_default()].join("_")
    }
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

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransferData {
    pub input_type: GemTransactionInputType,
    pub recipient: GemRecipient,
    pub value: GemBigInt,
    #[uniffi(default = false)]
    pub use_max_amount: bool,
    #[uniffi(default = None)]
    pub minimum_value: Option<GemBigInt>,
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmDestination {
    Recipient { name: Option<String>, address: String },
    Contract { address: String },
    Validator { name: String, address: String },
    Resource { resource: Resource },
    Provider { name: String, address: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_identifier_separates_transfers_by_chain_recipient_and_value() {
        let transfer = |address: &str, value: i32| GemTransferData {
            input_type: GemTransactionInputType::Transfer {
                asset: primitives::Asset::from_chain(primitives::Chain::Bitcoin),
            },
            recipient: GemRecipient::address(address.to_string()),
            value: value.into(),
            use_max_amount: false,
            minimum_value: None,
        };

        assert_eq!(transfer("bc1q", 10).identifier(), "bitcoin-bc1q-10");
        assert_ne!(transfer("bc1q", 10).identifier(), transfer("bc1q", 11).identifier());
        assert_ne!(transfer("bc1q", 10).identifier(), transfer("bc1r", 10).identifier());
    }

    #[test]
    fn test_recipient_identifier_separates_a_named_recipient_from_a_bare_address() {
        let recipient = |name: Option<&str>, address: &str, memo: Option<&str>| GemRecipient {
            address: address.to_string(),
            name: name.map(str::to_string),
            memo: memo.map(str::to_string),
            references: Vec::new(),
        };

        assert_eq!(recipient(Some("Alice"), "0xabc", Some("order 7")).identifier(), "Alice_0xabc_order 7");
        assert_eq!(recipient(None, "0xabc", None).identifier(), "_0xabc_");
        assert_ne!(recipient(Some("Alice"), "0xabc", None).identifier(), recipient(None, "0xabc", None).identifier());
    }
}
