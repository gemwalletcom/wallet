use crate::formatted_number::{GemFormattedNumber, GemNumberNotation, GemValueTone};
use crate::models::custom_types::{GemBigInt, GemBigUint};
use crate::models::list::{GemInfoTopic, GemListRow, GemListRowTitle};
use crate::precision::GemValueStyle;
use crate::services::assets::icon::{GemAssetIcon, asset_icon};
use crate::services::assets::model::{GemRowText, GemValueHeader};
use crate::services::localization::GemLocalizedText;
use crate::services::swap::model::GemSwapRate;
use chrono::{DateTime, Utc};
use primitives::{AddressName, Asset, AssetId, AssetPrice, Chain, NFTAssetId, PerpetualDirection, Resource, Transaction, TransactionDirection, TransactionId, TransactionListItem, TransactionState, TransactionType, TransactionsFilter};

use super::rules;
use crate::services::empty_state::GemEmptyStateKind;
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
pub fn activity_filters(chains: Vec<Chain>, filters: Vec<GemTransactionFilter>) -> TransactionsFilter {
    rules::activity_filters(chains, filters)
}

#[uniffi::export]
pub fn pending_activity_filters() -> TransactionsFilter {
    rules::pending_activity_filters()
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChainsFilterSummary {
    All,
    Chain { chain: Chain },
    Count { count: u32 },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemTransactionsFilterSummary {
    All,
    Filter { filter: GemTransactionFilter },
    Count { count: u32 },
}

#[uniffi::export]
pub fn chains_filter_summary(chains: Vec<Chain>) -> GemChainsFilterSummary {
    match chains.as_slice() {
        [] => GemChainsFilterSummary::All,
        [chain] => GemChainsFilterSummary::Chain { chain: *chain },
        selected => GemChainsFilterSummary::Count { count: selected.len() as u32 },
    }
}

#[uniffi::export]
pub fn transactions_filter_summary(filters: Vec<GemTransactionFilter>) -> GemTransactionsFilterSummary {
    match filters.as_slice() {
        [] => GemTransactionsFilterSummary::All,
        [filter] => GemTransactionsFilterSummary::Filter { filter: *filter },
        selected => GemTransactionsFilterSummary::Count { count: selected.len() as u32 },
    }
}

#[cfg(test)]
mod filter_summary_tests {
    use super::*;

    #[test]
    fn test_a_filter_reads_as_all_its_one_choice_or_a_count() {
        assert_eq!(chains_filter_summary(vec![]), GemChainsFilterSummary::All);
        assert_eq!(chains_filter_summary(vec![Chain::Ethereum]), GemChainsFilterSummary::Chain { chain: Chain::Ethereum });
        assert_eq!(chains_filter_summary(vec![Chain::Ethereum, Chain::Bitcoin, Chain::Solana]), GemChainsFilterSummary::Count { count: 3 });

        assert_eq!(transactions_filter_summary(vec![]), GemTransactionsFilterSummary::All);
        assert_eq!(transactions_filter_summary(vec![GemTransactionFilter::Swaps]), GemTransactionsFilterSummary::Filter { filter: GemTransactionFilter::Swaps });
        assert_eq!(transactions_filter_summary(vec![GemTransactionFilter::Swaps, GemTransactionFilter::Stake]), GemTransactionsFilterSummary::Count { count: 2 });
    }
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

impl GemAmountSign {
    pub fn amount(&self, value: &GemBigUint, asset: &Asset, style: GemValueStyle) -> GemFormattedNumber {
        let number = GemFormattedNumber::asset_amount(&GemBigInt::from(value.clone()), asset, style);
        match self {
            Self::None => number,
            _ if number.value == 0.0 => number,
            Self::Incoming => GemFormattedNumber {
                notation: GemNumberNotation::Signed,
                tone: GemValueTone::Positive,
                ..number
            },
            Self::Outgoing => GemFormattedNumber {
                value: -number.value,
                notation: GemNumberNotation::Signed,
                tone: GemValueTone::Plain,
                ..number
            },
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemHeaderAmount {
    pub asset: Asset,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub icon: GemAssetIcon,
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
    pub badge: GemTransactionBadge,
    pub icon: crate::services::assets::icon::GemAssetIcon,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTransactionBadge {
    Incoming,
    Outgoing,
    Asset,
}

#[uniffi::export]
pub fn transactions_empty_state(chains: Vec<Chain>, filters: Vec<GemTransactionFilter>) -> GemEmptyStateKind {
    match chains.is_empty() && filters.is_empty() {
        true => GemEmptyStateKind::Activity,
        false => GemEmptyStateKind::SearchActivity,
    }
}

#[uniffi::export]
pub fn transaction_rows(items: Vec<TransactionListItem>) -> Vec<GemTransactionRow> {
    items.iter().map(rules::row).collect()
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
#[allow(clippy::large_enum_variant)]
pub enum GemTransactionHeader {
    Amount { header: GemValueHeader },
    Value { header: GemValueHeader },
    Swap { from: GemHeaderAmount, to: GemHeaderAmount },
    Nft { name: Option<String>, image_url: String },
    AssetImage { icon: GemAssetIcon },
}

impl GemTransactionHeader {
    pub fn amount(amount: GemHeaderAmount) -> Self {
        Self::Amount {
            header: GemValueHeader::asset(
                amount.icon,
                GemLocalizedText::Number { number: amount.amount },
                amount.fiat.map(|fiat| GemRowText::neutral(GemLocalizedText::Number { number: fiat })),
            ),
        }
    }

    pub fn symbol(asset: &Asset) -> Self {
        Self::Amount {
            header: GemValueHeader::asset(asset_icon(&asset.id), GemLocalizedText::Text { text: asset.symbol.clone() }, None),
        }
    }

    pub fn asset_image(asset: &Asset) -> Self {
        Self::AssetImage { icon: asset_icon(&asset.id) }
    }
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
    Participant,
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
    pub provider_contract: Option<String>,
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
    pub amount: GemFormattedNumber,
    pub network: String,
    pub provider_name: String,
    pub transfer: GemSwapProgressState,
    pub swap: GemSwapProgressState,
    pub eta_seconds: Option<u32>,
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
    fn test_a_signed_amount_carries_its_direction_and_an_unsigned_one_does_not() {
        let asset = Asset {
            decimals: 2,
            ..Asset::from_chain(Chain::Bitcoin)
        };
        let number = |sign: GemAmountSign, value: u32| sign.amount(&value.into(), &asset, GemValueStyle::Auto);

        let incoming = number(GemAmountSign::Incoming, 100);
        assert_eq!((incoming.value, incoming.notation, incoming.tone), (1.0, GemNumberNotation::Signed, GemValueTone::Positive));

        let outgoing = number(GemAmountSign::Outgoing, 100);
        assert_eq!((outgoing.value, outgoing.notation, outgoing.tone), (-1.0, GemNumberNotation::Signed, GemValueTone::Plain));

        let unsigned = number(GemAmountSign::None, 100);
        assert_eq!((unsigned.value, unsigned.notation), (1.0, GemNumberNotation::Plain));

        let zero = number(GemAmountSign::Incoming, 0);
        assert_eq!(zero.notation, GemNumberNotation::Plain, "nothing moved, so nothing is signed");
    }

    #[test]
    fn test_a_full_amount_keeps_every_digit_past_what_a_double_holds() {
        let value = GemBigUint::from(123_456_789_012_345_678_901u128);
        let full = GemAmountSign::Outgoing.amount(&value, &Asset::from_chain(Chain::Ethereum), GemValueStyle::Full);
        assert_eq!((full.exact.as_deref(), full.value < 0.0), (Some("123.456789012345678901"), true), "the exact digits carry no sign; the value does");

        let auto = GemAmountSign::None.amount(&value, &Asset::from_chain(Chain::Ethereum), GemValueStyle::Auto);
        assert_eq!(auto.exact, None, "a rounded amount has no exact digits to keep");
    }
}
