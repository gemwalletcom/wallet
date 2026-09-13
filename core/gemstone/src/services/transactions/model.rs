use crate::models::custom_types::GemBigUint;
use crate::services::swap::model::GemSwapRate;
use primitives::{AddressName, Asset, AssetId, AssetPrice, NFTAssetId, PerpetualDirection, Resource, TransactionExtended, TransactionType};

use super::rules;
use primitives::BlockExplorerLink;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, uniffi::Enum)]
pub enum GemTransactionFilter {
    Transfers,
    Swaps,
    Stake,
    SmartContract,
    Perpetuals,
    Others,
}

#[uniffi::export]
impl GemTransactionFilter {
    pub fn transaction_types(&self) -> Vec<TransactionType> {
        rules::filter_transaction_types(*self)
    }
}

#[uniffi::export]
pub fn transaction_filters() -> Vec<GemTransactionFilter> {
    rules::transaction_filters()
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionTitle {
    Received,
    Sent,
    Transfer,
    SmartContract,
    Swap,
    Approve,
    Stake,
    Unstake,
    Redelegate,
    Rewards,
    Withdraw,
    ActivateAsset,
    Freeze,
    Unfreeze,
    Earn,
    PerpetualOpen { direction: Option<PerpetualDirection> },
    PerpetualClose { direction: Option<PerpetualDirection> },
    PerpetualModify,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GemTransactionSubtitle {
    None,
    ToAddress { address: String },
    FromAddress { address: String },
    ToResource { resource: Resource },
    FromResource { resource: Resource },
    Price { value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemAmountSign {
    None,
    Incoming,
    Outgoing,
}

#[uniffi::export]
impl GemAmountSign {
    pub fn format(&self, amount: String) -> String {
        match self {
            Self::None => amount,
            Self::Incoming => format!("+{amount}"),
            Self::Outgoing => format!("-{amount}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GemTransactionValue {
    None,
    AssetSymbol,
    Amount { sign: GemAmountSign },
    SwapReceived,
    SwapSpent,
    PerpetualNotional,
    PerpetualPnl { value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTransactionParticipantRole {
    Sender,
    Recipient,
    Contract,
    Validator,
    Provider,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionParticipant {
    pub role: GemTransactionParticipantRole,
    pub address: String,
    pub name: Option<AddressName>,
    pub link: BlockExplorerLink,
    pub can_add_contact: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionAmount {
    pub asset: Asset,
    pub value: GemBigUint,
    pub sign: GemAmountSign,
    pub price: Option<AssetPrice>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionRowSubtitle {
    None,
    ToAddress { participant: String },
    FromAddress { participant: String },
    ToResource { resource: Resource },
    FromResource { resource: Resource },
    Price { value: f64 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionRowValue {
    None,
    AssetSymbol { asset: Asset },
    Amount { amount: GemTransactionAmount },
    Fiat { value: f64 },
    Pnl { value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemTransactionStateTone {
    Pending,
    Success,
    Error,
    Refunded,
}

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Record)]
pub struct GemTransactionStatus {
    pub tone: GemTransactionStateTone,
    pub shows_badge: bool,
    pub shows_progress: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionRow {
    pub status: GemTransactionStatus,
    pub title: GemTransactionTitle,
    pub subtitle: GemTransactionRowSubtitle,
    pub value: GemTransactionRowValue,
    pub equivalent_value: GemTransactionRowValue,
    pub nft_image_url: Option<String>,
}

#[uniffi::export]
pub fn transactions_list_limit() -> u32 {
    primitives::TRANSACTIONS_LIMIT as u32
}

#[uniffi::export]
pub fn transaction_row(transaction: TransactionExtended) -> GemTransactionRow {
    rules::row(&transaction)
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionHeader {
    Amount { amount: GemTransactionAmount, shows_fiat: bool },
    Swap { from: GemTransactionAmount, to: GemTransactionAmount },
    Nft { asset_id: NFTAssetId, name: Option<String>, image_url: String },
    Symbol { asset: Asset },
    AssetImage { asset: Asset },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionHeaderAction {
    Asset { asset_id: AssetId },
    Nft { asset_id: NFTAssetId },
    Swap { from_asset_id: AssetId, to_asset_id: AssetId },
    Perpetual { asset_id: AssetId },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTransactionDetailRow {
    Header,
    SwapProgress,
    SwapAgain,
    Date,
    Status,
    EstimatedConfirmation,
    Participant,
    Memo,
    Resource,
    Rate,
    Network,
    Provider,
    Pnl,
    Price,
    Fee,
    Explorer,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionDetailSection {
    pub rows: Vec<GemTransactionDetailRow>,
}

#[uniffi::export]
pub fn transaction_detail_sections(rows: GemTransactionDetailRows) -> Vec<GemTransactionDetailSection> {
    rules::detail_sections(&rows)
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionDetailRows {
    pub status: GemTransactionStatus,
    pub title: GemTransactionTitle,
    pub header: GemTransactionHeader,
    pub header_action: Option<GemTransactionHeaderAction>,
    pub swap_progress: Option<GemSwapProgress>,
    pub swap_again: Option<GemSwapAgain>,
    pub estimated_confirmation_seconds: Option<u32>,
    pub participant: Option<GemTransactionParticipant>,
    pub provider_name: Option<String>,
    pub memo: Option<String>,
    pub resource: Option<Resource>,
    pub rate: Option<GemSwapRate>,
    pub pnl: Option<f64>,
    pub price: Option<f64>,
    pub fee: GemTransactionAmount,
    pub explorer: BlockExplorerLink,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionHeaderKind {
    Amount { shows_fiat: bool },
    Swap,
    Nft,
    Symbol,
    AssetImage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GemTransactionDetails {
    pub swap_progress: Option<GemSwapProgress>,
    pub swap_again: Option<GemSwapAgain>,
    pub provider_name: Option<String>,
    pub estimated_confirmation_seconds: Option<u32>,
    pub pnl: Option<f64>,
    pub price: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapProgress {
    pub from_asset: Asset,
    pub from_value: GemBigUint,
    pub provider_name: String,
    pub transfer: GemSwapProgressStep,
    pub swap: GemSwapProgressStep,
    pub eta_seconds: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapProgressStep {
    Pending,
    Waiting,
    Completed,
    Failed,
    Reverted,
    Refunded,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapAgain {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

#[cfg(test)]
mod tests {
    use super::GemAmountSign;

    #[test]
    fn test_a_signed_amount_carries_its_direction_and_an_unsigned_one_does_not() {
        assert_eq!(GemAmountSign::Incoming.format("1.00 BTC".to_string()), "+1.00 BTC");
        assert_eq!(GemAmountSign::Outgoing.format("1.00 BTC".to_string()), "-1.00 BTC");
        assert_eq!(GemAmountSign::None.format("1.00 BTC".to_string()), "1.00 BTC");
    }
}
