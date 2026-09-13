use super::rules;
use crate::services::balance::GemBalanceRequirement;
use crate::services::swap::GemAssetRate;
use primitives::{FiatProviderName, FiatTransactionStatus};

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
    pub rate: Option<GemAssetRate>,
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
