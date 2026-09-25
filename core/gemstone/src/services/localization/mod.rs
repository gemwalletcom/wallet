use primitives::{AddressType, Chain, DelegationState, FeeUnitType, PerpetualDirection, PerpetualMarginType, Resource, StakeProviderType, TransactionState};

use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::GemFormattedNumber;

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
    Pnl { amount: GemFormattedNumber, percent: GemFormattedNumber },
    Margin { amount: GemFormattedNumber, margin_type: PerpetualMarginType },
    Position { direction: PerpetualDirection, leverage: GemFormattedNumber },
    Apr { value: Option<GemFormattedNumber> },
    PriceImpactWarning { percent: GemFormattedNumber, symbol: String },
    Balance { amount: GemFormattedNumber },
    StakeProvider { provider: StakeProviderType },
    NftCollections,
    NftUnverified,
    SignInWith { chain: Chain },
    ReviewRequest,
    EnableDeveloper,
    RewardsRedeemAsset { value: GemFormattedNumber },
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
