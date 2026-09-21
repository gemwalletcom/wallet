use chrono::{DateTime, Utc};
use primitives::{Asset, BlockExplorerLink, Chain, TransactionState};

use crate::config::social::GemSocialLink;
use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::copy::GemCopy;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::service_status::GemLatencyStatus;
use crate::services::transactions::GemTransactionStateTone;
use crate::services::wallet::model::GemWalletRow;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemListSectionTitle {
    None,
    Balances,
    Community,
    Manage,
    Resources,
    SocialLinks,
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
    MarketPrice,
    EntryPrice,
    LiquidationPrice,
    Margin,
    FundingPayments,
    MarketCap,
    FullyDilutedValuation,
    TradingVolume,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemNoticeKind {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemInfoTopic {
    NetworkFee { asset: Asset },
    OpenInterest,
    FundingApr,
    StakeApr,
    StakeLockTime,
    TransactionStatus { state: TransactionState, tone: GemTransactionStateTone },
    AutoClose,
    LiquidationPrice,
    FundingPayments,
    FullyDilutedValuation,
    CirculatingSupply,
    TotalSupply,
    MaxSupply,
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
    Amount {
        title: GemListRowTitle,
        amount: GemFormattedNumber,
        info: Option<GemInfoTopic>,
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
    },
    Picker {
        title: GemListRowTitle,
        value: GemLocalizedText,
        icon: GemListRowIcon,
    },
    Social {
        links: Vec<GemSocialLink>,
    },
    Icon {
        chain: Chain,
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemListSection {
    pub title: GemListSectionTitle,
    pub footer: GemListSectionFooter,
    pub rows: Vec<GemListRow>,
}
