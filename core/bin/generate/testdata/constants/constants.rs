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
pub const LIMITS: GemAttachmentLimits = GemAttachmentLimits { jpeg_quality: 90, max_dimension: 2048 };
const REJECTED_CODE: i32 = 4001;
const INTERNAL: u8 = 7;

pub fn rejected_error() -> GemRpcError {
    GemRpcError {
        code: REJECTED_CODE,
        message: "User rejected the request".to_string(),
    }
}

pub fn labels() -> Vec<GemLabel> {
    vec![GemLabel::None, GemLabel::Number { value: 2.0, digits: Some(1) }, GemLabel::Plain("text".into())]
}

fn private_helper() -> u8 {
    INTERNAL
}
