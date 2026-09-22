use crate::models::custom_types::GemBigInt;
use crate::models::transaction::{GemTransactionLoadFee, GemTransactionLoadMetadata};
use primitives::TransactionInputType;
use primitives::{AssetId, PerpetualDirection, RecentActivityType, Resource, SimulationResult, TransactionType, TransferDataOutputAction, TransferDataOutputType};

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
        [self.input_type.get_asset().chain().as_ref(), &self.recipient.address, &self.value.to_string()].join("-")
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
        Self { name: Some(name), ..Self::address(address) }
    }
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct GemTransferData {
    pub input_type: TransactionInputType,
    pub recipient: GemRecipient,
    pub value: GemBigInt,
    #[uniffi(default = false)]
    pub use_max_amount: bool,
}

#[derive(Debug, Clone, PartialEq)]
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
pub enum GemConfirmTitle {
    Send,
    Deposit,
    Withdraw,
    Swap,
    Approve,
    Request,
    Payment,
    Stake,
    Unstake,
    Redelegate,
    ClaimRewards,
    Freeze,
    Unfreeze,
    ActivateAsset,
    PerpetualOpen { direction: PerpetualDirection },
    PerpetualIncrease { direction: PerpetualDirection },
    PerpetualReduce { direction: PerpetualDirection },
    PerpetualClose,
    PerpetualModify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GemConfirmRow {
    App,
    Sender,
    Recipient,
    Network,
    Memo,
    Details,
    PaymentAsset,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemConfirmDestination {
    Recipient { name: Option<String>, address: String },
    Contract { name: Option<String>, address: String },
    Validator { name: String, address: String },
    Resource { resource: Resource },
    Provider { name: String, address: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_identifier_separates_transfers_by_chain_recipient_and_value() {
        let transfer = GemTransferData {
            recipient: GemRecipient::address("bc1q".to_string()),
            value: 10.into(),
            ..GemTransferData::mock(TransactionInputType::Transfer {
                asset: primitives::Asset::from_chain(primitives::Chain::Bitcoin),
            })
        };

        assert_eq!(transfer.identifier(), "bitcoin-bc1q-10");
        assert_ne!(transfer.identifier(), GemTransferData { value: 11.into(), ..transfer.clone() }.identifier());
        assert_ne!(
            transfer.identifier(),
            GemTransferData {
                recipient: GemRecipient::address("bc1r".to_string()),
                ..transfer.clone()
            }
            .identifier()
        );
    }

    #[test]
    fn test_recipient_identifier_separates_a_named_recipient_from_a_bare_address() {
        let named = GemRecipient::named("0xabc".to_string(), "Alice".to_string());

        assert_eq!(
            GemRecipient {
                memo: Some("order 7".to_string()),
                ..named.clone()
            }
            .identifier(),
            "Alice_0xabc_order 7"
        );
        assert_eq!(GemRecipient::address("0xabc".to_string()).identifier(), "_0xabc_");
        assert_ne!(named.identifier(), GemRecipient::address("0xabc".to_string()).identifier());
    }
}
