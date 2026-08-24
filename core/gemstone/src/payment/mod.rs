mod error;

use std::sync::Arc;

use payment::{PaymentService, WalletConnectPayAuth};
use primitives::{ChainAddress, PaymentLink, PaymentURLDecoder, WALLET_CONNECT_PAY_HOST};

use crate::GemstoneError;
use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::models::payment::{GemPayment, GemPaymentOptions, GemPaymentOutcome, GemPaymentQuote, GemPaymentQuoteData};
use error::PaymentError;

pub type GemWalletConnectPayAuth = WalletConnectPayAuth;

#[uniffi::remote(Record)]
pub struct GemWalletConnectPayAuth {
    pub app_id: String,
    pub client_id: String,
}

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<GemPayment, GemstoneError> {
    Ok(PaymentURLDecoder::decode(string)?)
}

#[uniffi::export]
pub fn payment_wallet_connect_url() -> String {
    format!("https://{WALLET_CONNECT_PAY_HOST}")
}

#[derive(uniffi::Object)]
pub struct GemPaymentService {
    service: PaymentService,
}

#[uniffi::export]
impl GemPaymentService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>, wallet_connect_pay: GemWalletConnectPayAuth) -> Self {
        Self {
            service: PaymentService::new(Arc::new(AlienProviderWrapper::new(provider)), wallet_connect_pay),
        }
    }

    pub async fn get_options(&self, link: PaymentLink, addresses: Vec<ChainAddress>) -> Result<GemPaymentOptions, PaymentError> {
        self.service.get_options(&link, &addresses).await
    }

    pub async fn get_quote_data(&self, quote: GemPaymentQuote, addresses: Vec<ChainAddress>) -> Result<GemPaymentQuoteData, PaymentError> {
        self.service.get_quote_data(&quote, &addresses).await
    }

    pub async fn confirm(&self, quote: GemPaymentQuote, transaction_hash: String) -> Result<GemPaymentOutcome, PaymentError> {
        self.service.confirm(&quote, transaction_hash).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::payment::{GemPaymentAmount, GemPaymentLink, GemPaymentRequest};
    use primitives::AssetId;
    use primitives::Chain;

    #[test]
    fn test_request() {
        assert_eq!(
            payment_decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            GemPayment::Request(GemPaymentRequest {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some(GemPaymentAmount::ExactValue("0.42301".to_string())),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
            })
        );
    }

    #[test]
    fn test_link() {
        assert_eq!(
            payment_decode_url("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1").unwrap(),
            GemPayment::Link(GemPaymentLink::SolanaPay("https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1".to_string()))
        );
        assert_eq!(
            payment_decode_url("https://pay.walletconnect.com/?pid=pay_123").unwrap(),
            GemPayment::Link(GemPaymentLink::WalletConnectPay("pay_123".to_string()))
        );
    }
}
