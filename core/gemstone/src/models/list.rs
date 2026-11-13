use chrono::{DateTime, Utc};
use primitives::{AddressFormatStyle, Asset, BlockExplorerLink, Chain, TransactionState, VerificationStatus};
use strum::AsRefStr;

use crate::address_formatter::format_address;
use crate::config::social::GemSocialLink;
use crate::constants::LIST_PAGE_SIZE;
use crate::duration_formatter::GemDurationPart;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::models::copy::{GemCopy, GemCopyKind, address_copy};
use crate::models::custom_types::GemBigInt;
use crate::services::assets::icon::GemAssetIcon;
use crate::models::state::GemListPhase;
use crate::services::contact::model::GemAvatar;
use crate::services::currency::GemCurrencyRow;
use crate::services::stake::GemValidatorRow;
use crate::services::price_alert::rules::GemPriceAlertItem;
use crate::services::wallet_connect::GemConnection;
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
    Recommended,
    All,
    Active,
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
    Withdraw,
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
    HideBalance,
    Currency,
    Language,
    Appearance,
    Networks,
    Contacts,
    Perpetuals,
    PerpetualLeverage,
    PerpetualMarginType,
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
    PriorityFee,
    BaseFee,
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
    Unverified,
    RewardsPending,
    Warning,
    SuspiciousAddress,
    UnlimitedApproval,
    NftCollectionApproval,
    SuspiciousAsset,
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
    Method,
    Token,
    Spender,
    Recipient,
    Value,
    Expiration,
    TokenId,
    Collection,
}

pub(crate) fn suspicious_address_title(kind: GemNoticeKind) -> GemListRowTitle {
    match kind {
        GemNoticeKind::Error => GemListRowTitle::SuspiciousAddress,
        GemNoticeKind::Warning | GemNoticeKind::Info => GemListRowTitle::Warning,
    }
}

pub(crate) fn asset_status_notice(status: VerificationStatus) -> Option<GemListRow> {
    let (title, kind) = match status {
        VerificationStatus::Unverified => (GemListRowTitle::Warning, GemNoticeKind::Warning),
        VerificationStatus::Suspicious => (GemListRowTitle::SuspiciousAsset, GemNoticeKind::Error),
        VerificationStatus::Verified => return None,
    };
    Some(GemListRow::Notice {
        title,
        message: Some(GemLocalizedText::SuspiciousAssetDescription),
        kind,
    })
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
    RegionUnavailable,
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
    StakeLockup { remaining: Vec<GemDurationPart> },
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
    PriceAlertsAuto,
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

#[derive(Debug, Clone, PartialEq, Eq, AsRefStr, uniffi::Enum)]
#[strum(serialize_all = "camelCase")]
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
    PerpetualMarginType,
    PerpetualTakeProfit,
    PerpetualStopLoss,
    PushNotifications,
    Authentication,
    LockPeriod,
    HideBalance,
    PriceAlerts,
    SetPriceAlert,
    Price,
    Network,
    Earn,
    Stake,
    Pin,
    AddToWallet,
    SelectWallet,
    UnverifiedCollections,
    Explorer { url: String },
}

#[uniffi::export]
impl GemRowAction {
    pub fn identifier(&self) -> String {
        self.as_ref().to_string()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemListFieldValue {
    Text {
        text: String,
    },
    Date {
        date: DateTime<Utc>,
    },
    Address {
        name: Option<String>,
        copy: GemCopy,
        explorer: BlockExplorerLink,
        menu: Vec<GemRowMenuItem>,
    },
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
    Verification {
        title: GemListRowTitle,
        status: VerificationStatus,
        info: Option<GemInfoTopic>,
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
        action: Option<GemRowAction>,
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
    Field {
        title: GemLocalizedText,
        value: GemListFieldValue,
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

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemProviderRow {
    pub kind: GemProviderKind,
    pub name: String,
    pub amount: GemFormattedNumber,
    pub fiat: Option<GemFormattedNumber>,
    pub is_selected: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemProviderLoadingRow {
    pub kind: GemProviderKind,
    pub name: String,
}

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
        let short_address = format_address(&address, Some(chain), AddressFormatStyle::Short);
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

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemRowMenuItem {
    Copy { copy: GemCopy },
    Open { title: GemLocalizedText, url: String },
}

impl GemRowMenuItem {
    pub(crate) fn view_on(explorer: &BlockExplorerLink) -> Self {
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
            action: None,
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
            address: match copy.kind {
                GemCopyKind::Address { .. } => Some(copy.value.clone()),
                GemCopyKind::Plain | GemCopyKind::SecretPhrase | GemCopyKind::PrivateKey => None,
            },
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
            Self::Wallet { action, .. } => action.clone(),
            _ => None,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemListItem {
    Row { row: GemListRow },
    Currency { row: GemCurrencyRow },
    Validator { row: GemValidatorRow, is_selected: bool },
    Connection { connection: GemConnection },
    PriceAlert { item: GemPriceAlertItem },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemGroup {
    pub id: Option<String>,
    pub title: Option<GemLocalizedText>,
    pub footer: GemListSectionFooter,
    pub items: Vec<GemListItem>,
}

impl GemGroup {
    pub fn new(title: GemListSectionTitle, items: Vec<GemListItem>) -> Self {
        match title {
            GemListSectionTitle::None => Self {
                id: None,
                title: None,
                footer: GemListSectionFooter::None,
                items,
            },
            title => Self::with_title(format!("{title:?}"), Some(GemLocalizedText::SectionTitle { title }), items),
        }
    }

    pub fn section(title: GemListSectionTitle, rows: Vec<GemListRow>) -> Self {
        Self::new(title, rows.into_iter().map(|row| GemListItem::Row { row }).collect())
    }

    pub fn untitled(rows: Vec<GemListRow>) -> Self {
        Self::section(GemListSectionTitle::None, rows)
    }

    pub fn named(id: String, name: String, items: Vec<GemListItem>) -> Self {
        Self::with_title(id, Some(GemLocalizedText::Text { text: name }), items)
    }

    pub fn with_title(id: String, title: Option<GemLocalizedText>, items: Vec<GemListItem>) -> Self {
        Self {
            id: Some(id),
            title,
            footer: GemListSectionFooter::None,
            items,
        }
    }

    pub fn with_footer(self, footer: GemListSectionFooter) -> Self {
        Self { footer, ..self }
    }

    #[cfg(test)]
    pub fn rows(&self) -> Vec<GemListRow> {
        self.items
            .iter()
            .filter_map(|item| match item {
                GemListItem::Row { row } => Some(row.clone()),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemGroupList {
    pub groups: Vec<GemGroup>,
    pub phase: GemListPhase,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemListSection {
    pub title: GemListSectionTitle,
    pub footer: GemListSectionFooter,
    pub rows: Vec<GemListRow>,
}

impl GemListSection {
    pub fn new(title: GemListSectionTitle, rows: Vec<GemListRow>) -> Self {
        Self {
            title,
            footer: GemListSectionFooter::None,
            rows,
        }
    }

    pub fn untitled(rows: Vec<GemListRow>) -> Self {
        Self::new(GemListSectionTitle::None, rows)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemListWindow {
    pub limit: usize,
    pub maximum: usize,
}

#[uniffi::export]
impl GemListWindow {
    pub fn on_end_reached(&self, row_count: usize) -> GemListWindow {
        match row_count >= self.limit {
            true => Self {
                limit: (self.limit + LIST_PAGE_SIZE).min(self.maximum),
                ..*self
            },
            false => *self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_on_end_reached() {
        let first = GemListWindow {
            limit: LIST_PAGE_SIZE,
            maximum: LIST_PAGE_SIZE * 3,
        };

        assert_eq!(first.on_end_reached(LIST_PAGE_SIZE), GemListWindow { limit: LIST_PAGE_SIZE * 2, ..first });
        assert_eq!(first.on_end_reached(LIST_PAGE_SIZE - 1), first);
        assert_eq!(
            GemListWindow {
                limit: LIST_PAGE_SIZE * 2,
                maximum: LIST_PAGE_SIZE * 2 + 1
            }
            .on_end_reached(LIST_PAGE_SIZE * 2),
            GemListWindow {
                limit: LIST_PAGE_SIZE * 2 + 1,
                maximum: LIST_PAGE_SIZE * 2 + 1
            }
        );
        assert_eq!(GemListWindow { limit: LIST_PAGE_SIZE * 3, ..first }.on_end_reached(LIST_PAGE_SIZE * 3), GemListWindow { limit: LIST_PAGE_SIZE * 3, ..first });
    }

    #[test]
    fn test_row_action_identifier() {
        assert_eq!(GemRowAction::PriceAlerts.identifier(), "priceAlerts");
        assert_eq!(GemRowAction::Explorer { url: "https://example.com".to_string() }.identifier(), "explorer");
    }
}
