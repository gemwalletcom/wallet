use crate::models::custom_types::GemBigUint;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatAmountCheck {
    BelowMinimum { minimum: u32 },
    AboveMaximum { maximum: u32 },
    InsufficientBalance { required: GemBigUint, available: GemBigUint },
    Valid,
}
