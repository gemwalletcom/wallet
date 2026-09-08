use crate::services::balance::GemBalanceRequirement;
use primitives::FiatProviderName;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatAmountCheck {
    BelowMinimum { minimum: u32 },
    AboveMaximum { maximum: u32 },
    InsufficientBalance { requirement: GemBalanceRequirement },
    Valid,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatQuoteRow {
    pub quote_id: String,
    pub provider: FiatProviderName,
    pub provider_name: String,
    pub provider_image_url: Option<String>,
    pub crypto_amount: f64,
    pub fiat_amount: f64,
    pub rate: Option<f64>,
}
