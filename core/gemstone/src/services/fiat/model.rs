use crate::services::balance::GemBalanceRequirement;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatAmountCheck {
    BelowMinimum { minimum: u32 },
    AboveMaximum { maximum: u32 },
    InsufficientBalance { requirement: GemBalanceRequirement },
    Valid,
}
