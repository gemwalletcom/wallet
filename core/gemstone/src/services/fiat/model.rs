use super::rules;
use crate::formatted_number::{GemFormattedNumber, GemValueTone};
use crate::services::swap::GemAssetRate;
use primitives::{FiatProviderName, FiatQuoteType, FiatTransactionAssetData};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemFiatAmountCheck {
    BelowMinimum { minimum: GemFormattedNumber },
    AboveMaximum { maximum: GemFormattedNumber },
    InsufficientBalance { title: String },
    Valid,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatSuggestedAmount {
    pub amount: u32,
    pub value: GemFormattedNumber,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatQuoteRow {
    pub quote_id: String,
    pub provider: FiatProviderName,
    pub provider_name: String,
    pub crypto_amount: GemFormattedNumber,
    pub fiat_amount: GemFormattedNumber,
    pub rate: Option<GemAssetRate>,
}

#[uniffi::export]
impl GemFiatQuoteRow {
    pub fn crypto_estimate_text(&self, formatted_value: String) -> String {
        format!("≈ {formatted_value}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemFiatTransactionBadge {
    Pending,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Record)]
pub struct GemFiatTransactionStatus {
    pub badge: Option<GemFiatTransactionBadge>,
    pub tone: GemValueTone,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatTransactionRow {
    pub quote_type: FiatQuoteType,
    pub provider: FiatProviderName,
    pub subtitle: String,
    pub value: GemFormattedNumber,
    pub fiat_value: GemFormattedNumber,
    pub badge: Option<GemFiatTransactionBadge>,
    pub details_url: Option<String>,
}

#[uniffi::export]
pub fn fiat_transaction_rows(data: Vec<FiatTransactionAssetData>) -> Vec<GemFiatTransactionRow> {
    data.iter().map(rules::transaction_row).collect()
}
