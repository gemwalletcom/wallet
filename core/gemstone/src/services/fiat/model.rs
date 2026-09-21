use super::rules;
use crate::formatted_number::GemFormattedNumber;
use crate::services::swap::GemAssetRate;
use primitives::{FiatProviderName, FiatQuoteType, FiatTransactionAssetData, FiatTransactionStatus};

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
    pub is_dimmed: bool,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemFiatTransactionRow {
    pub quote_type: FiatQuoteType,
    pub provider: FiatProviderName,
    pub subtitle: String,
    pub value: GemFormattedNumber,
    pub fiat_value: GemFormattedNumber,
    pub badge: Option<GemFiatTransactionBadge>,
    pub is_dimmed: bool,
    pub details_url: Option<String>,
}

#[uniffi::export]
pub fn fiat_transaction_row(data: FiatTransactionAssetData) -> GemFiatTransactionRow {
    rules::transaction_row(&data)
}

#[uniffi::export]
pub fn fiat_transaction_status(status: FiatTransactionStatus) -> GemFiatTransactionStatus {
    rules::transaction_status(status)
}

#[uniffi::export]
pub fn fiat_provider_name(provider: FiatProviderName) -> String {
    provider.name().to_string()
}

#[cfg(test)]
mod provider_tests {
    use super::*;

    #[test]
    fn test_a_provider_reads_by_its_brand_and_not_its_case_name() {
        assert_eq!(fiat_provider_name(FiatProviderName::Flashnet), "Cash App");
        assert_eq!(fiat_provider_name(FiatProviderName::MoonPay), "MoonPay");
    }
}
