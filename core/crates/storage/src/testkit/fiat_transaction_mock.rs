use crate::models::FiatTransactionRow;
use primitives::{AssetId, Chain, DateTime, FiatProviderName, FiatQuoteType, FiatTransactionStatus};

impl FiatTransactionRow {
    pub fn mock() -> Self {
        Self {
            id: 1,
            asset_id: AssetId::from_chain(Chain::Ethereum).into(),
            transaction_type: FiatQuoteType::Buy.into(),
            provider_id: FiatProviderName::MoonPay.into(),
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending.into(),
            country: Some("US".to_string()),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            value: Some("123000000000000000".to_string()),
            address_id: 1,
            transaction_hash: Some("0xabc".to_string()),
            device_id: 1,
            wallet_id: 1,
            quote_id: "quote_123".to_string(),
            updated_at: DateTime::UNIX_EPOCH.into(),
            created_at: DateTime::UNIX_EPOCH.into(),
        }
    }

    pub fn mock_with_timestamps(created_at: DateTime, updated_at: DateTime) -> Self {
        Self {
            created_at: created_at.into(),
            updated_at: updated_at.into(),
            ..Self::mock()
        }
    }

    pub fn mock_without_value() -> Self {
        Self { value: None, ..Self::mock() }
    }
}
