use chrono::{DateTime, Utc};
use primitives::{Asset, BlockExplorerLink, Chain, TransactionState, VerificationStatus};

use crate::address_formatter::{GemAddressFormatStyle, format_address};
use crate::config::social::GemSocialLink;
use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::copy::{GemCopy, address_copy};
use crate::models::custom_types::GemBigInt;
use crate::services::assets::icon::GemAssetIcon;
use crate::services::contact::model::GemAvatar;
use crate::services::error::GemServiceError;
use crate::services::localization::GemLocalizedText;
use crate::services::service_status::GemLatencyStatus;
use crate::services::swap::GemAssetRate;
use crate::services::transactions::GemTransactionStateTone;
use crate::services::transfer::GemRecipient;
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
    NormalFee,
    FastFee,
    CustomFee,
    PayWith,
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
    EnablePriceAlerts,
    Deposit,
    NoData,
    Connection,
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
    App,
    Memo,
    Transfer,
    Swap,
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
        tag: String,
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
        title: GemListRowTitle,
        name: String,
        icon_url: Option<String>,
        menu: Vec<GemRowMenuItem>,
    },
    Wallet {
        title: GemListRowTitle,
        wallet: GemWalletRow,
        menu: Vec<GemRowMenuItem>,
    },
    Memo {
        title: GemListRowTitle,
        value: String,
        menu: Vec<GemRowMenuItem>,
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
        label: GemLocalizedText,
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
    AssetChange {
        name: String,
        icon: crate::services::assets::icon::GemAssetIcon,
        amount: GemFormattedNumber,
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
        title: GemLocalizedText,
        url: String,
    },
    Identifier {
        title: GemListRowTitle,
        copy: GemCopy,
        explorer: Option<BlockExplorerLink>,
        address: Option<String>,
        menu: Vec<GemRowMenuItem>,
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemProviderKind {
    Swap { provider: swapper::SwapperProvider },
    Fiat { provider: primitives::FiatProviderName },
}

/// A swap or fiat provider offering an amount: the name, what it gives and its value, and whether it is the one picked.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemProviderRow {
    pub kind: GemProviderKind,
    pub name: String,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub is_selected: bool,
}

/// An address a screen shows: the text it reads, the short address a tap reveals when the text is a name, and its long-press menu.
#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAddressRow {
    pub title: GemLocalizedText,
    pub text: GemLocalizedText,
    pub short_address: Option<String>,
    pub chain: Chain,
    pub address: String,
    pub avatar: Option<GemAvatar>,
    pub menu: Vec<GemRowMenuItem>,
    pub contact: Option<GemRecipient>,
    pub is_selectable: bool,
}

impl GemAddressRow {
    pub fn new(title: GemLocalizedText, chain: Chain, address: String, name: Option<&str>, explorer: &BlockExplorerLink) -> Self {
        let short_address = format_address(&address, Some(chain), GemAddressFormatStyle::Short);
        let name = name.filter(|name| !name.is_empty() && *name != address);
        Self {
            title,
            text: GemLocalizedText::Text {
                text: name.map(str::to_string).unwrap_or_else(|| short_address.clone()),
            },
            short_address: name.map(|_| short_address),
            chain,
            menu: match address.is_empty() {
                true => vec![],
                false => vec![GemRowMenuItem::Copy { copy: address_copy(chain, address.clone()) }, GemRowMenuItem::view_on(explorer)],
            },
            address,
            avatar: None,
            contact: None,
            is_selectable: false,
        }
    }
}

/// An entry of a row's long-press menu; a copy entry reads "Copy" on both apps.
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemRowMenuItem {
    Copy { copy: GemCopy },
    Open { title: GemLocalizedText, url: String },
}

impl GemRowMenuItem {
    fn view_on(explorer: &BlockExplorerLink) -> Self {
        Self::Open {
            title: GemLocalizedText::ViewOn { name: explorer.name.clone() },
            url: explorer.link.clone(),
        }
    }
}

impl GemListRow {
    pub fn app(name: String, icon_url: Option<String>, website_url: Option<String>) -> Self {
        Self::App {
            title: GemListRowTitle::App,
            name,
            icon_url,
            menu: website_url
                .map(|url| GemRowMenuItem::Open {
                    title: GemLocalizedText::RowTitle { title: GemListRowTitle::Website },
                    url,
                })
                .into_iter()
                .collect(),
        }
    }

    pub fn wallet(wallet: GemWalletRow, copy: GemCopy, explorer: BlockExplorerLink) -> Self {
        Self::Wallet {
            title: GemListRowTitle::Wallet,
            wallet,
            menu: vec![GemRowMenuItem::Copy { copy }, GemRowMenuItem::view_on(&explorer)],
        }
    }

    pub fn memo(value: String, copy: Option<String>) -> Self {
        Self::Memo {
            title: GemListRowTitle::Memo,
            value,
            menu: copy.map(|value| GemRowMenuItem::Copy { copy: GemCopy::plain(value) }).into_iter().collect(),
        }
    }

    pub fn identifier(title: GemListRowTitle, copy: GemCopy, explorer: Option<BlockExplorerLink>) -> Self {
        Self::Identifier {
            address: (title == GemListRowTitle::Contract).then(|| copy.value.clone()),
            menu: [Some(GemRowMenuItem::Copy { copy: copy.clone() }), explorer.as_ref().map(GemRowMenuItem::view_on)].into_iter().flatten().collect(),
            title,
            copy,
            explorer,
        }
    }

    pub fn explorer(explorer: &BlockExplorerLink) -> Self {
        Self::Explorer {
            title: GemLocalizedText::ViewOn { name: explorer.name.clone() },
            url: explorer.link.clone(),
        }
    }

    pub fn ranked(title: GemListRowTitle, amount: GemFormattedNumber, rank: i32) -> Self {
        Self::Ranked { title, amount, tag: format!("#{rank}") }
    }

    pub fn toggle(title: GemListRowTitle, icon: GemListRowIcon, is_on: bool, action: GemRowAction) -> Self {
        Self::Toggle {
            label: GemLocalizedText::RowTitle { title },
            icon,
            is_on,
            action,
        }
    }
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
