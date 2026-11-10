use crate::currency::Currency;
use crate::fiat_assets::{FiatAsset, FiatAssetLimits};
use crate::{Asset, AssetId, Chain, FiatProvider, FiatProviderName, FiatQuote, FiatQuoteRequest, FiatQuoteType, FiatTransaction, FiatTransactionStatus, PaymentType};
use chrono::{DateTime, Utc};
use num_bigint::BigUint;
use std::collections::HashMap;

impl FiatQuoteRequest {
    pub fn mock() -> Self {
        FiatQuoteRequest {
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            quote_type: FiatQuoteType::Buy,
            currency: "USD".to_string(),
            amount: 100.0,
            provider_id: None,
            ip_address: "192.168.1.1".to_string(),
        }
    }

    pub fn mock_sell() -> Self {
        FiatQuoteRequest {
            quote_type: FiatQuoteType::Sell,
            amount: 250.0,
            ..Self::mock()
        }
    }
}

impl FiatProvider {
    pub fn mock(id: FiatProviderName) -> Self {
        FiatProvider {
            id,
            name: id.name().to_string(),
            image_url: None,
            priority: None,
            threshold_bps: None,
            enabled: true,
            buy_enabled: true,
            sell_enabled: true,
            payment_methods: vec![],
        }
    }

    pub fn mock_with_priority(id: FiatProviderName, priority: i32, threshold_bps: Option<i32>) -> Self {
        FiatProvider {
            priority: Some(priority),
            threshold_bps,
            ..Self::mock(id)
        }
    }
}

impl FiatQuote {
    pub fn mock(provider_id: FiatProviderName) -> Self {
        FiatQuote {
            id: "quote_123".to_string(),
            asset: Asset::from_chain(Chain::Bitcoin),
            provider: FiatProvider::mock(provider_id),
            quote_type: FiatQuoteType::Buy,
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            crypto_amount: 0.001,
            value: BigUint::from(100_000u64),
            latency: 0,
            payment_methods: vec![PaymentType::Card],
        }
    }
}

impl FiatTransaction {
    pub fn mock() -> Self {
        FiatTransaction {
            id: "quote_123".to_string(),
            asset_id: AssetId::from_chain(Chain::Bitcoin),
            transaction_type: FiatQuoteType::Buy,
            provider: FiatProviderName::MoonPay,
            provider_transaction_id: Some("tx_123".to_string()),
            status: FiatTransactionStatus::Pending,
            country: Some("US".to_string()),
            fiat_amount: 100.0,
            fiat_currency: "USD".to_string(),
            value: BigUint::from(100_000u64),
            transaction_hash: None,
            created_at: DateTime::<Utc>::UNIX_EPOCH,
            updated_at: DateTime::<Utc>::UNIX_EPOCH,
        }
    }
}

impl FiatAsset {
    pub fn mock(enabled: bool) -> Self {
        FiatAsset {
            id: "gram".to_string(),
            asset_id: Some(AssetId::from_chain(Chain::Ton)),
            provider: FiatProviderName::MoonPay,
            symbol: "gram".to_string(),
            network: Some("ton".to_string()),
            token_id: None,
            enabled,
            is_buy_enabled: true,
            is_sell_enabled: true,
            unsupported_countries: HashMap::new(),
            buy_limits: vec![],
            sell_limits: vec![],
        }
    }
}

impl FiatAssetLimits {
    pub fn mock() -> Self {
        FiatAssetLimits {
            currency: Currency::USD,
            payment_type: PaymentType::Card,
            min_amount: Some(50.0),
            max_amount: Some(10000.0),
        }
    }
}
