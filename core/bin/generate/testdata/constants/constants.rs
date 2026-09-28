use std::time::Duration;

use primitives::{Currency, TransactionType};

pub const SEARCH_DEBOUNCE: Duration = Duration::from_millis(250);
pub const SCAN_TIMEOUT: Duration = Duration::from_secs(3);
pub const RECENT_ASSETS_LIMIT: usize = 10;
pub const MAX_ATTEMPTS: u32 = 5;
pub const OFFSET: i64 = -2;
pub const RATIO: f64 = 0.5;
pub const ENABLED: bool = true;
pub const GREETING: &str = "Say \"hi\" for $5";
pub const EMOJIS: &[&str] = &["💎", "🦄"];
pub const LOCK_PERIODS: &[GemLockPeriod] = &[GemLockPeriod::Immediate, GemLockPeriod::OneMinute];
pub const ACTIVITY_TYPES: &[TransactionType] = &[TransactionType::TransferNFT, TransactionType::Swap];
pub const QUOTE_CURRENCY: Currency = Currency::USD;
const INTERNAL: u8 = 7;
