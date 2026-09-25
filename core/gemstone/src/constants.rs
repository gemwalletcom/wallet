use std::time::Duration;

use primitives::{BannerEvent, Currency, TransactionType};

use crate::services::onboarding::{GemAcceptTermsItem, GemSecurityReminderItem};
use crate::services::security::GemLockPeriod;
use crate::services::transactions::GemTransactionFilter;

pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(250);
pub const NODE_CHECK_DEBOUNCE: Duration = Duration::from_millis(250);
pub const SWAP_QUOTE_DEBOUNCE: Duration = Duration::from_millis(250);
pub const SWAP_QUOTE_REFRESH_INTERVAL: Duration = Duration::from_secs(30);
pub const FIAT_QUOTE_DEBOUNCE: Duration = Duration::from_millis(250);
pub const FIAT_QUOTE_REFRESH_INTERVAL: Duration = Duration::from_secs(300);
pub const FIAT_QUOTE_CURRENCY: Currency = Currency::USD;
pub const CONNECTION_BANNER_SETTLE_DELAY: Duration = Duration::from_secs(10);
pub const OFFLINE_DEBOUNCE: Duration = Duration::from_millis(500);
pub const PING_INTERVAL: Duration = Duration::from_secs(30);
pub const SCAN_TIMEOUT: Duration = Duration::from_secs(3);
pub const SERVICE_STATUS_TIMEOUT: Duration = Duration::from_secs(30);
pub const TRANSACTIONS_LIST_LIMIT: usize = 1000;
pub const ASSET_RESULTS_LIMIT: usize = 100;
pub const RECENT_ASSETS_LIMIT: usize = 10;
pub const SUPPORT_ATTACHMENT_MAX_DIMENSION: usize = 2048;
pub const SUPPORT_ATTACHMENT_JPEG_QUALITY: usize = 90;
pub const WALLET_CONNECT_USER_REJECTED_ERROR_CODE: i32 = 4001;
pub const WALLET_CONNECT_USER_REJECTED_ERROR_MESSAGE: &str = "User rejected the request";
pub const WALLET_BANNER_EVENTS: &[BannerEvent] = &[BannerEvent::AccountBlockedMultiSignature, BannerEvent::Onboarding];
pub const PERPETUAL_ACTIVITY_TYPES: &[TransactionType] = &[TransactionType::PerpetualOpenPosition, TransactionType::PerpetualClosePosition, TransactionType::PerpetualModifyPosition];
pub const ACCEPT_TERMS_ITEMS: &[GemAcceptTermsItem] = &[GemAcceptTermsItem::SelfCustody, GemAcceptTermsItem::Recovery, GemAcceptTermsItem::Responsibility];
pub const SECURITY_REMINDER_ITEMS: &[GemSecurityReminderItem] = &[GemSecurityReminderItem::KeepSafe, GemSecurityReminderItem::DoNotShare, GemSecurityReminderItem::NoRecovery];
pub const TRANSACTION_FILTERS: &[GemTransactionFilter] = &[
    GemTransactionFilter::Transfers,
    GemTransactionFilter::Swaps,
    GemTransactionFilter::Stake,
    GemTransactionFilter::SmartContract,
    GemTransactionFilter::Perpetuals,
    GemTransactionFilter::Others,
];
pub const LOCK_PERIODS: &[GemLockPeriod] = &[
    GemLockPeriod::Immediate,
    GemLockPeriod::OneMinute,
    GemLockPeriod::FiveMinutes,
    GemLockPeriod::FifteenMinutes,
    GemLockPeriod::OneHour,
    GemLockPeriod::SixHours,
];
pub const WALLET_AVATAR_EMOJIS: &[&str] = &[
    "💎",
    "🦄",
    "🚀",
    "❤️",
    "😍",
    "🔥",
    "💩",
    "😭",
    "🏆",
    "🏴‍☠️",
    "✅",
    "⚠️",
    "💰",
    "🎁",
    "🎈",
    "🌈",
    "⭐️",
    "👑",
    "💔",
    "🔒",
    "🏦",
    "🥷",
    "👨‍💻",
    "🛢",
    "🔑",
    "🛡",
    "📈",
    "📉",
    "💥",
    "👽",
    "🔮",
    "⚡️",
    "🌍",
    "⏳",
    "🤖",
    "🛰",
    "🐉",
    "🐙",
    "🦅",
    "👀",
    "💪",
    "🔷",
    "👻",
    "🌪",
    "🕶",
    "👾",
    "🕵️‍♂️",
    "⌛️",
    "✨",
    "🍀",
    "☠️",
    "💀",
    "🕸",
    "🕷",
    "🎰",
    "☄️",
    "🏔",
    "🏜",
    "🌊",
    "🎆",
    "🎖",
    "🔭",
    "⛽️",
    "🏭",
    "🌉",
    "🏰",
    "🔨",
    "🧰",
    "💼",
    "🏷",
    "♟",
    "⚓️",
    "🎡",
    "🎢",
    "🎃",
    "📦",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limits_shared_with_other_crates_stay_in_step() {
        assert_eq!(TRANSACTIONS_LIST_LIMIT, primitives::TRANSACTIONS_LIMIT, "the apps list as many transactions as the API pages");
        assert_eq!(SERVICE_STATUS_TIMEOUT, gem_client::DEFAULT_REQUEST_TIMEOUT, "the status check waits as long as any request");
    }

    #[test]
    fn test_onboarding_asks_for_every_term_and_shows_every_reminder() {
        assert_eq!(ACCEPT_TERMS_ITEMS.len(), 3, "a wallet is only created once all three terms are accepted");
        assert_eq!(SECURITY_REMINDER_ITEMS.first(), Some(&GemSecurityReminderItem::KeepSafe));
        assert_eq!(SECURITY_REMINDER_ITEMS.last(), Some(&GemSecurityReminderItem::NoRecovery));
    }

    #[test]
    fn test_the_avatar_list_leads_with_the_gem_and_has_no_duplicates() {
        assert_eq!(WALLET_AVATAR_EMOJIS.first(), Some(&"\u{1f48e}"));
        let unique: std::collections::HashSet<&&str> = WALLET_AVATAR_EMOJIS.iter().collect();
        assert_eq!(unique.len(), WALLET_AVATAR_EMOJIS.len());
    }
}
