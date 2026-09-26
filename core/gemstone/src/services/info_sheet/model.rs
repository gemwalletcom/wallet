use primitives::{Asset, SwapProvider, TransactionState, VerificationStatus};

use crate::formatted_number::GemFormattedNumber;
use crate::services::assets::icon::GemAssetIcon;
use crate::services::confirm::GemAcquireAsset;
use crate::services::transactions::model::GemTransactionStateTone;

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemInfoSheet {
    pub title: GemInfoTitle,
    pub description: GemInfoDescription,
    pub image: GemInfoImage,
    pub action: Option<GemInfoAction>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoTitle {
    NetworkFee,
    BalanceRequired { symbol: String },
    TransactionState { state: TransactionState },
    EstimatedConfirmation,
    WatchWallet,
    PaymentVerification,
    LockTime,
    Apr,
    PriceImpact,
    Slippage,
    NoQuote,
    AssetStatus { status: VerificationStatus },
    AccountMinimumBalance,
    MinimumAmount,
    StakingReservedFees,
    Pending,
    StakeFrozenRequired,
    FundingApr,
    FundingPayments,
    LiquidationPrice,
    OpenInterest,
    AutoClose,
    MaliciousTransaction,
    Warning,
    TransferError,
    FullyDilutedValuation,
    CirculatingSupply,
    TotalSupply,
    MaxSupply,
    WalletName { name: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemInfoAmount {
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoDescription {
    NetworkFee {
        network: String,
        symbol: String,
    },
    BalanceRequired {
        required: Option<GemFormattedNumber>,
        available: Option<GemFormattedNumber>,
        shortfall: Option<GemFormattedNumber>,
    },
    InsufficientNetworkFeeBalance {
        required: Option<GemInfoAmount>,
        network: String,
        available: GemFormattedNumber,
        shortfall: Option<GemFormattedNumber>,
    },
    InsufficientNetworkFee {
        title: String,
    },
    TransactionState {
        tone: GemTransactionStateTone,
    },
    EstimatedConfirmation {
        network: String,
    },
    WatchWallet,
    PaymentVerification,
    LockTime,
    Apr,
    PriceImpact,
    Slippage,
    NoQuote,
    AssetStatus {
        status: VerificationStatus,
    },
    AccountMinimumBalance {
        amount: Option<GemFormattedNumber>,
    },
    MinimumAmount {
        network: String,
        amount: GemFormattedNumber,
    },
    SwapMinimumAmount {
        provider: String,
        required: Option<GemInfoAmount>,
        available: Option<GemFormattedNumber>,
        shortfall: Option<GemFormattedNumber>,
    },
    StakingReservedFees,
    Pending,
    StakeFrozenRequired,
    FundingApr,
    FundingPayments,
    LiquidationPrice,
    OpenInterest,
    AutoClose,
    MaliciousTransaction,
    MemoRequired {
        symbol: String,
    },
    DustThreshold {
        network: String,
    },
    FullyDilutedValuation,
    CirculatingSupply,
    TotalSupply,
    MaxSupply,
    ExistingWalletImported,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoImage {
    Logo,
    NetworkFee,
    WatchWallet,
    AssetStatus { status: VerificationStatus },
    SwapProvider { provider: SwapProvider },
    Asset { icon: GemAssetIcon },
    TransactionState { icon: GemAssetIcon, tone: GemTransactionStateTone },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoAction {
    LearnMore { url: String },
    Buy { symbol: String },
    Acquire { asset: Asset, acquire: GemAcquireAsset },
    Continue,
}
