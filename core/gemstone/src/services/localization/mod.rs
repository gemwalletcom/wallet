use primitives::{AddressType, Chain, DelegationState, FeeUnitType, PerpetualDirection, PerpetualMarginType, Resource, StakeProviderType, TransactionState};

use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::GemFormattedNumber;
use crate::services::perpetual::model::GemPerpetualChartLineKind;
use crate::services::price_alert::rules::GemPriceAlertLabel;
use crate::services::wallet::model::GemWalletSecretKind;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemLocalizedText {
    WalletDefaultName { index: i32 },
    WalletDefaultNameChain { network_name: String, index: i32 },
    WalletMulticoin,
    ChainNetworkName { chain: Chain },
    DelegationState { state: DelegationState },
    TransactionState { state: TransactionState },
    Resource { resource: Resource },
    FeeRate { rate: GemFormattedNumber, unit: FeeUnitType },
    Text { text: String },
    RowTitle { title: crate::models::list::GemListRowTitle },
    EnableValue { value: String },
    ViewOn { name: String },
    AmountOnNetwork { amount: GemFormattedNumber, network: String },
    ParticipantRole { role: crate::services::transactions::GemTransactionParticipantRole },
    ConfirmDestination { destination: crate::services::transfer::GemConfirmDestination },
    Number { number: GemFormattedNumber },
    None,
    SlippageAuto,
    RewardsUnverified,
    RewardsPending { countdown: Vec<GemDurationPart> },
    RewardsPendingReady,
    ErrorOccurred,
    UnlimitedApprovalWarning,
    ExternallyOwnedSpenderWarning,
    SuspiciousAddressDescription,
    AddressType { address_type: AddressType },
    InvalidTokenId,
    TriggerOrder { order: GemTriggerOrder, price: Option<GemFormattedNumber> },
    ChartLine { kind: GemPerpetualChartLineKind, price: GemFormattedNumber },
    ExpectedProfit,
    ExpectedLoss,
    ShowSecret { kind: GemWalletSecretKind },
    SecretKind { kind: GemWalletSecretKind },
    NewWallet,
    NewTag,
    DeleteConfirmation { name: String },
    Pinned { name: String, pinned: bool },
    PriceAlertsToggled { name: String, enabled: bool },
    AmountBalance { balance: GemFormattedNumber },
    ReservedFees { fee: GemFormattedNumber },
    RewardsInviteDescription { points: GemFormattedNumber },
    RewardsShareText { link: String },
    CurrentPrice { price: GemFormattedNumber },
    Pnl { amount: GemFormattedNumber, percent: GemFormattedNumber },
    Margin { amount: GemFormattedNumber, margin_type: PerpetualMarginType },
    Position { direction: PerpetualDirection, leverage: GemFormattedNumber },
    PriceAlertLabel { label: GemPriceAlertLabel },
    Apr { value: Option<GemFormattedNumber> },
    PriceImpactWarning { percent: GemFormattedNumber, symbol: String },
    Balance { amount: GemFormattedNumber },
    AvailableBalance { amount: GemFormattedNumber },
    UnlimitedAsset { symbol: String },
    StakeProvider { provider: StakeProviderType },
    NftCollections,
    SelectAsset,
    NftUnverified,
    SignInWith { chain: Chain },
    ReviewRequest,
    EnableDeveloper,
    RewardsRedeemAsset { value: GemFormattedNumber },
    RewardsConfirmRedeem { value: GemFormattedNumber, points: GemFormattedNumber },
    PriceAlertAddedPriceOver { value: GemFormattedNumber },
    PriceAlertAddedPriceUnder { value: GemFormattedNumber },
    PriceAlertAddedIncreasesBy { value: GemFormattedNumber },
    PriceAlertAddedDecreasesBy { value: GemFormattedNumber },
    DisableDeveloper,
    PositionChange { change: GemPositionChange, direction: PerpetualDirection },
    PerpetualConfirmed { action: GemPerpetualConfirmedAction },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemPerpetualConfirmedAction {
    Open { direction: PerpetualDirection },
    Close,
    Modify,
    Increase,
    Reduce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemPositionChange {
    Increase,
    Reduce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemTriggerOrder {
    TakeProfit,
    StopLoss,
}
