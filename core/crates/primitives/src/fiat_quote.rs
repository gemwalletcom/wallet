use crate::{Asset, FiatQuoteType, PaymentType, fiat_provider::FiatProvider};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatQuote {
    pub id: String,
    pub asset: Asset,
    pub provider: FiatProvider,
    #[serde(rename = "type")]
    pub quote_type: FiatQuoteType,
    pub fiat_amount: f64,
    pub fiat_currency: String,
    pub crypto_amount: f64,
    #[serde(default)]
    pub value: BigUint,
    #[serde(default)]
    pub latency: u64,
    pub payment_methods: Vec<PaymentType>,
}

impl FiatQuote {
    pub fn new(
        id: String,
        asset: Asset,
        provider: FiatProvider,
        quote_type: FiatQuoteType,
        fiat_amount: f64,
        fiat_currency: String,
        crypto_amount: f64,
        value: BigUint,
        latency: u64,
        payment_methods: Vec<PaymentType>,
    ) -> Self {
        Self {
            id,
            asset,
            provider,
            quote_type,
            fiat_amount,
            fiat_currency,
            crypto_amount,
            value,
            latency,
            payment_methods,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatQuotes {
    pub quotes: Vec<FiatQuote>,
    pub errors: Vec<FiatQuoteError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteUrl {
    pub redirect_url: String,
    #[serde(skip_serializing)]
    pub provider_transaction_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiatQuoteError {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    pub error: String,
}

impl FiatQuoteError {
    pub fn new(provider: Option<String>, error: String) -> Self {
        Self { provider, error }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatQuoteResponse {
    pub quote_id: String,
    pub fiat_amount: f64,
    pub crypto_amount: f64,
    pub payment_methods: Vec<PaymentType>,
}

impl FiatQuoteResponse {
    pub fn new(quote_id: String, fiat_amount: f64, crypto_amount: f64) -> Self {
        Self {
            quote_id,
            fiat_amount,
            crypto_amount,
            payment_methods: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FiatAssetSymbol {
    pub symbol: String,
    pub network: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FiatQuoteUrlData {
    pub quote: FiatQuote,
    pub asset_symbol: FiatAssetSymbol,
    pub wallet_address: String,
    pub ip_address: String,
    pub locale: String,
}
