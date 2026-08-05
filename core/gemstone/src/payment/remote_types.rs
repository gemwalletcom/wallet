use chrono::{DateTime, Utc};

use crate::GemstoneError;
use primitives::payment_decoder::wallet_connect_pay::WALLET_CONNECT_PAY_HOST;
use primitives::{
    AssetId, Payment, PaymentAmount, PaymentLink, PaymentMerchant, PaymentOptions, PaymentOutcome, PaymentPrice, PaymentProviderName, PaymentQuote, PaymentQuotes, PaymentRequest,
    PaymentStatus, PaymentURLDecoder,
};

pub type GemPayment = Payment;
pub type GemPaymentRequest = PaymentRequest;
pub type GemPaymentLink = PaymentLink;
pub type GemPaymentProviderName = PaymentProviderName;
pub type GemPaymentMerchant = PaymentMerchant;
pub type GemPaymentOutcome = PaymentOutcome;
pub type GemPaymentStatus = PaymentStatus;
pub type GemPaymentOptions = PaymentOptions;
pub type GemPaymentQuotes = PaymentQuotes;
pub type GemPaymentPrice = PaymentPrice;
pub type GemPaymentQuote = PaymentQuote;
pub type GemPaymentAmount = PaymentAmount;

#[uniffi::remote(Enum)]
pub enum GemPaymentOptions {
    Quotes(GemPaymentQuotes),
    Outcome(GemPaymentOutcome),
}

#[uniffi::remote(Record)]
pub struct GemPaymentQuotes {
    pub merchant: GemPaymentMerchant,
    pub price: Option<GemPaymentPrice>,
    pub expires_at: Option<DateTime<Utc>>,
    pub quotes: Vec<GemPaymentQuote>,
}

#[uniffi::remote(Record)]
pub struct GemPaymentQuote {
    pub id: String,
    pub payment_id: String,
    pub amount: GemPaymentAmount,
    pub expires_at: Option<DateTime<Utc>>,
    pub collect_data_url: Option<String>,
    pub provider_data: String,
}

#[uniffi::remote(Record)]
pub struct GemPaymentAmount {
    pub asset_id: AssetId,
    pub value: String,
    pub symbol: String,
    pub decimals: i32,
}

#[uniffi::remote(Record)]
pub struct GemPaymentMerchant {
    pub name: String,
    pub icon_url: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentStatus {
    RequiresAction,
    Processing,
    Succeeded,
    Failed,
    Expired,
    Cancelled,
}

#[uniffi::remote(Record)]
pub struct GemPaymentOutcome {
    pub status: GemPaymentStatus,
    pub transaction_id: Option<String>,
}

#[uniffi::remote(Enum)]
pub enum GemPayment {
    Request(GemPaymentRequest),
    Link(GemPaymentLink),
}

#[uniffi::remote(Record)]
pub struct GemPaymentRequest {
    pub address: String,
    pub amount: Option<String>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
}

#[uniffi::remote(Record)]
pub struct GemPaymentLink {
    pub provider: PaymentProviderName,
    pub id: String,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentProviderName {
    SolanaPay,
    WalletConnectPay,
}

#[uniffi::export]
pub fn payment_wallet_connect_url() -> String {
    format!("https://{WALLET_CONNECT_PAY_HOST}")
}

#[uniffi::export]
pub fn payment_provider_has_status(provider: GemPaymentProviderName) -> bool {
    provider.has_status()
}

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<GemPayment, GemstoneError> {
    Ok(PaymentURLDecoder::decode(string)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    #[test]
    fn test_request() {
        assert_eq!(
            payment_decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            GemPayment::Request(GemPaymentRequest {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some("0.42301".to_string()),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
            })
        );
    }

    #[test]
    fn test_link() {
        assert_eq!(
            payment_decode_url("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1").unwrap(),
            GemPayment::Link(GemPaymentLink::new(
                PaymentProviderName::SolanaPay,
                "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1".to_string()
            ))
        );
        assert_eq!(
            payment_decode_url("https://pay.walletconnect.com/?pid=pay_123").unwrap(),
            GemPayment::Link(GemPaymentLink::new(PaymentProviderName::WalletConnectPay, "pay_123".to_string()))
        );
    }
}

#[uniffi::remote(Record)]
pub struct GemPaymentPrice {
    pub symbol: String,
    pub value: String,
    pub decimals: i32,
}
