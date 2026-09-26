use chrono::{DateTime, Utc};
use primitives::{Asset, BlockExplorerLink, Chain, TransactionState, VerificationStatus};

use crate::config::social::GemSocialLink;
use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::copy::GemCopy;
use crate::models::custom_types::GemBigInt;
use crate::services::assets::icon::GemAssetIcon;
use crate::services::contact::model::GemAvatar;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::service_status::GemLatencyStatus;
use crate::services::swap::GemAssetRate;
use crate::services::transactions::GemTransactionStateTone;
use crate::services::wallet::model::{GemWalletPlaceholder, GemWalletRow};

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListSectionTitle {
    None,
    Balances,
    Info,
    Community,
    Manage,
    Resources,
    SocialLinks,
    Properties,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListRowTitle {
    Api,
    Stream,
    GemWalletNode,
    Name,
    Network,
    Address,
    Available,
    Stake,
    Freeze,
    Unfreeze,
    ClaimRewards,
    Earn,
    PendingUnconfirmed,
    Reserved,
    Error,
    TermsOfService,
    PrivacyPolicy,
    Website,
    Version,
    UpdateApp,
    Wallets,
    Security,
    Notifications,
    Preferences,
    WalletConnect,
    Support,
    Rewards,
    MyReferralCode,
    Referrals,
    Points,
    InvitedBy,
    AboutUs,
    Developer,
    Authentication,
    LockPeriod,
    PrivacyLock,
    HideBalance,
    Currency,
    Language,
    Appearance,
    Networks,
    Contacts,
    Perpetuals,
    PerpetualLeverage,
    PerpetualTakeProfit,
    PerpetualStopLoss,
    DailyVolume,
    OpenInterest,
    FundingApr,
    StakeApr,
    LockTime,
    MinimumAmount,
    NetworkFee,
    Validator,
    Provider,
    Status,
    ActiveIn,
    AvailableIn,
    Date,
    Resource,
    Price,
    Pnl,
    Pin,
    Unpin,
    AddToWallet,
    PriceAlerts,
    SetPriceAlert,
    Energy,
    Bandwidth,
    RewardsUnverified,
    RewardsPending,
    Warning,
    SuspiciousAddress,
    UnlimitedApproval,
    NftCollectionApproval,
    Symbol,
    Decimals,
    Type,
    AutoClose,
    Size,
    Position,
    Details,
    Slippage,
    PriceImpact,
    MinimumReceive,
    EstimatedTime,
    EstimatedConfirmation,
    MarketPrice,
    Rate,
    EntryPrice,
    LiquidationPrice,
    Margin,
    FundingPayments,
    MarketCap,
    FullyDilutedValuation,
    TradingVolume,
    UnrealizedPnl,
    AccountLeverage,
    MarginUsage,
    AllTimePnl,
    Volume,
    CirculatingSupply,
    TotalSupply,
    MaxSupply,
    AllTimeHigh,
    AllTimeLow,
    Wallet,
    Contract,
    TokenId,
    Collection,
}

pub(crate) fn suspicious_address_title(kind: GemNoticeKind) -> GemListRowTitle {
    match kind {
        GemNoticeKind::Error => GemListRowTitle::SuspiciousAddress,
        GemNoticeKind::Warning | GemNoticeKind::Info => GemListRowTitle::Warning,
    }
}

pub(crate) fn suspicious_address_notice(kind: GemNoticeKind) -> GemListRow {
    GemListRow::Notice {
        title: suspicious_address_title(kind),
        message: Some(GemLocalizedText::SuspiciousAddressDescription),
        kind,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNoticeKind {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoTopic {
    NetworkFee { asset: Asset },
    PriceImpact,
    Slippage,
    MinimumAmount { asset: Asset, minimum: GemBigInt },
    NoQuote,
    OpenInterest,
    FundingApr,
    StakeApr { chain: Chain },
    StakeLockTime { chain: Chain },
    StakeFrozenRequired,
    TransactionStatus { state: TransactionState, tone: GemTransactionStateTone, icon: GemAssetIcon },
    AutoClose,
    LiquidationPrice,
    FundingPayments,
    FullyDilutedValuation,
    CirculatingSupply,
    TotalSupply,
    MaxSupply,
    EstimatedConfirmation { chain: Chain },
    WatchWallet,
    PaymentVerification,
    StakingReservedFees { asset: Asset },
    PendingUnconfirmedBalance,
    AssetStatus { status: VerificationStatus },
    ExistingWalletImported { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListSectionFooter {
    None,
    Authentication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemUrlTarget {
    InApp,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListRowIcon {
    None,
    AppLogo,
    Wallets,
    Security,
    Notifications,
    PriceAlerts,
    Preferences,
    WalletConnect,
    Support,
    Rewards,
    AboutUs,
    Developer,
    Currency,
    Language,
    Appearance,
    Networks,
    Contacts,
    Perpetuals,
    Pin,
    Unpin,
    AddToWallet,
}

/// What tapping a row opens or switches; each app maps it to its route or handler once.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Enum)]
pub enum GemRowAction {
    Wallets,
    Security,
    Notifications,
    Preferences,
    WalletConnect,
    Support,
    Rewards,
    AboutUs,
    Developer,
    Currency,
    Language,
    Appearance,
    Networks,
    Contacts,
    Perpetuals,
    PerpetualLeverage,
    PerpetualTakeProfit,
    PerpetualStopLoss,
    PushNotifications,
    Authentication,
    LockPeriod,
    PrivacyLock,
    HideBalance,
    PriceAlerts,
    SetPriceAlert,
    Price,
    Network,
    Earn,
    Stake,
    Pin,
    AddToWallet,
    Explorer { url: String },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemListRow {
    Latency {
        title: GemListRowTitle,
        title_suffix: String,
        host: String,
        status: GemLatencyStatus,
    },
    Notice {
        title: GemListRowTitle,
        message: Option<GemLocalizedText>,
        kind: GemNoticeKind,
    },
    Text {
        title: GemListRowTitle,
        value: String,
    },
    Provider {
        title: GemListRowTitle,
        name: String,
        contract: Option<String>,
    },
    Amount {
        title: GemListRowTitle,
        amount: GemFormattedNumber,
        info: Option<GemInfoTopic>,
    },
    Rate {
        title: GemListRowTitle,
        rate: GemAssetRate,
        inverse: Option<GemAssetRate>,
    },
    Action {
        title: GemListRowTitle,
        value: Option<GemFormattedNumber>,
        info: Option<GemInfoTopic>,
    },
    Quote {
        title: GemListRowTitle,
        value: Option<GemFormattedNumber>,
        change: Option<GemFormattedNumber>,
    },
    Ranked {
        title: GemListRowTitle,
        amount: GemFormattedNumber,
        rank: i32,
    },
    AllTime {
        title: GemListRowTitle,
        value: GemFormattedNumber,
        date: DateTime<Utc>,
        change: GemFormattedNumber,
    },
    Duration {
        title: GemListRowTitle,
        parts: Vec<GemDurationPart>,
        info: Option<GemInfoTopic>,
        estimate: bool,
    },
    Label {
        title: GemListRowTitle,
        text: GemLocalizedText,
        tone: GemValueTone,
        info: Option<GemInfoTopic>,
        progress: bool,
    },
    Date {
        title: GemListRowTitle,
        date: DateTime<Utc>,
    },
    Network {
        title: GemListRowTitle,
        chain: Chain,
        name: String,
    },
    App {
        name: String,
        icon_url: Option<String>,
        website_url: Option<String>,
    },
    Wallet {
        wallet: GemWalletRow,
        copy: GemCopy,
        explorer: BlockExplorerLink,
    },
    Memo {
        value: String,
        copy: Option<String>,
    },
    Link {
        title: GemListRowTitle,
        value: Option<String>,
        icon: GemListRowIcon,
        action: GemRowAction,
    },
    Url {
        title: GemListRowTitle,
        value: Option<String>,
        icon: GemListRowIcon,
        url: String,
        target: GemUrlTarget,
    },
    Toggle {
        title: GemListRowTitle,
        value: Option<String>,
        icon: GemListRowIcon,
        is_on: bool,
        action: GemRowAction,
    },
    Picker {
        title: GemListRowTitle,
        value: GemLocalizedText,
        icon: GemListRowIcon,
        action: GemRowAction,
    },
    Social {
        links: Vec<GemSocialLink>,
    },
    Icon {
        icon: crate::services::assets::icon::GemAssetIcon,
        image_url: Option<String>,
    },
    Avatar {
        avatar: GemAvatar,
    },
    WalletAvatar {
        image_url: Option<String>,
        placeholder: GemWalletPlaceholder,
    },
    Address {
        address: String,
        copy: GemCopy,
    },
    Explorer {
        name: String,
        url: String,
    },
    Identifier {
        title: GemListRowTitle,
        copy: GemCopy,
        explorer: Option<BlockExplorerLink>,
    },
    Lines {
        title: GemListRowTitle,
        lines: Vec<GemLocalizedText>,
        info: Option<GemInfoTopic>,
    },
    Loading,
    Error {
        error: GemServiceError,
    },
}

#[uniffi::export]
impl GemListRow {
    pub fn action(&self) -> Option<GemRowAction> {
        match self {
            Self::Link { action, .. } | Self::Toggle { action, .. } | Self::Picker { action, .. } => Some(action.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemListSection {
    pub title: GemListSectionTitle,
    pub footer: GemListSectionFooter,
    pub rows: Vec<GemListRow>,
}
