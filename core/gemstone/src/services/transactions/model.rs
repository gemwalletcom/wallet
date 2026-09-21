use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::custom_types::GemBigUint;
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::services::swap::model::GemSwapRate;
use chrono::{DateTime, Utc};
use primitives::{AddressName, Asset, AssetId, AssetPrice, Chain, ChainAsset, NFTAssetId, PerpetualDirection, Resource, Transaction, TransactionDirection, TransactionExtended, TransactionId, TransactionState, TransactionType};

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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemActivityFilters {
    pub asset_rank_greater_than: i32,
    pub chains: Vec<Chain>,
    pub transaction_types: Vec<TransactionType>,
}

#[uniffi::export]
pub fn activity_filters(chains: Vec<Chain>, filters: Vec<GemTransactionFilter>) -> GemActivityFilters {
    rules::activity_filters(chains, filters)
}

#[uniffi::export]
pub fn transaction_asset_ids(transaction: Transaction) -> Vec<AssetId> {
    transaction.asset_ids()
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
    pub text: String,
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
    Price { price: GemFormattedNumber },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionRowValue {
    None,
    AssetSymbol { asset: Asset },
    Number { number: GemFormattedNumber },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
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
    pub id: TransactionId,
    pub asset: Asset,
    pub transaction_type: TransactionType,
    pub direction: TransactionDirection,
    pub state: TransactionState,
    pub created_at: DateTime<Utc>,
    pub status: GemTransactionStatus,
    pub title: GemTransactionTitle,
    pub subtitle: GemTransactionRowSubtitle,
    pub value: GemTransactionRowValue,
    pub value_tone: GemValueTone,
    pub equivalent_value: GemTransactionRowValue,
    pub nft_image_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTransactionsEmptyState {
    NoActivity,
    NoResults,
}

#[uniffi::export]
pub fn transactions_empty_state(chains: Vec<Chain>, filters: Vec<GemTransactionFilter>) -> GemTransactionsEmptyState {
    match chains.is_empty() && filters.is_empty() {
        true => GemTransactionsEmptyState::NoActivity,
        false => GemTransactionsEmptyState::NoResults,
    }
}

#[uniffi::export]
pub fn transactions_list_limit() -> u32 {
    primitives::TRANSACTIONS_LIMIT as u32
}

#[uniffi::export]
pub fn transaction_row(transaction: TransactionExtended) -> GemTransactionRow {
    rules::row(&transaction)
}

#[uniffi::export]
pub fn transaction_rows(transactions: Vec<TransactionExtended>) -> Vec<GemTransactionRow> {
    transactions.iter().map(rules::row).collect()
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemTransactionDetailRow {
    Header,
    SwapProgress,
    SwapAgain,
    EstimatedConfirmation,
    Participant,
    Rate,
    Fee,
    Row { row: GemListRow },
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
    pub id: TransactionId,
    pub asset: Asset,
    pub transaction_type: TransactionType,
    pub direction: TransactionDirection,
    pub state: TransactionState,
    pub created_at: DateTime<Utc>,
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
    pub pnl: Option<GemFormattedNumber>,
    pub price: Option<GemFormattedNumber>,
    pub fee: GemTransactionAmount,
    pub fee_row: GemTransactionFeeRow,
    pub explorer: BlockExplorerLink,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemTransactionFeeRow {
    pub title: GemListRowTitle,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub info: GemInfoTopic,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionHeaderKind {
    Amount { shows_fiat: bool },
    Payment,
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
    pub transfer: GemSwapProgressState,
    pub swap: GemSwapProgressState,
    pub eta_seconds: Option<u32>,
}

#[uniffi::export]
impl GemSwapProgress {
    pub fn transfer_text(&self, formatted_value: String) -> String {
        format!("{formatted_value} ({})", ChainAsset::from_chain(self.from_asset.chain()).network_name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemSwapProgressState {
    pub step: GemSwapProgressStep,
    pub marker: GemSwapProgressMarker,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSwapProgressMarker {
    Check,
    Spinner,
    Dots,
    Cross,
    Swap,
}

impl GemSwapProgressStep {
    pub fn state(self) -> GemSwapProgressState {
        GemSwapProgressState { step: self, marker: self.marker() }
    }

    fn marker(self) -> GemSwapProgressMarker {
        match self {
            Self::Completed => GemSwapProgressMarker::Check,
            Self::Pending => GemSwapProgressMarker::Spinner,
            Self::Waiting => GemSwapProgressMarker::Dots,
            Self::Failed | Self::Reverted => GemSwapProgressMarker::Cross,
            Self::Refunded => GemSwapProgressMarker::Swap,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapAgain {
    pub from_asset_id: AssetId,
    pub to_asset_id: AssetId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_swap_transfer_names_the_network_beside_the_amount() {
        let progress = GemSwapProgress {
            from_asset: Asset::from_chain(Chain::Ethereum),
            from_value: GemBigUint::ZERO,
            provider_name: "Thorchain".to_string(),
            transfer: GemSwapProgressState {
                step: GemSwapProgressStep::Pending,
                marker: GemSwapProgressMarker::Spinner,
            },
            swap: GemSwapProgressState {
                step: GemSwapProgressStep::Waiting,
                marker: GemSwapProgressMarker::Spinner,
            },
            eta_seconds: None,
        };

        assert_eq!(progress.transfer_text("0.5 ETH".to_string()), "0.5 ETH (Ethereum)");
    }
    use super::GemAmountSign;

    #[test]
    fn test_a_signed_amount_carries_its_direction_and_an_unsigned_one_does_not() {
        assert_eq!(GemAmountSign::Incoming.format("1.00 BTC".to_string()), "+1.00 BTC");
        assert_eq!(GemAmountSign::Outgoing.format("1.00 BTC".to_string()), "-1.00 BTC");
        assert_eq!(GemAmountSign::None.format("1.00 BTC".to_string()), "1.00 BTC");
    }
}
