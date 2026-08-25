use std::sync::Arc;

use crate::GemstoneError;
use crate::alien::{AlienProvider, AlienProviderWrapper};
use crate::models::payment::{GemPayment, GemPaymentLink, GemPaymentTransaction};
use payment::PaymentService as CorePaymentService;
use primitives::{ChainAddress, PaymentURLDecoder};

pub type GemPaymentError = payment::PaymentError;

#[uniffi::remote(Enum)]
pub enum GemPaymentError {
    NoPaymentOptions,
    InvalidRequest { reason: String },
    Network { reason: String },
}

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<GemPayment, GemstoneError> {
    Ok(PaymentURLDecoder::decode(string)?)
}

#[derive(uniffi::Object)]
pub struct PaymentService {
    service: CorePaymentService,
}

#[uniffi::export]
impl PaymentService {
    #[uniffi::constructor]
    pub fn new(provider: Arc<dyn AlienProvider>) -> Self {
        Self {
            service: CorePaymentService::new(Arc::new(AlienProviderWrapper::new(provider))),
        }
    }

    pub async fn load(&self, link: GemPaymentLink, addresses: Vec<ChainAddress>) -> Result<GemPaymentTransaction, GemPaymentError> {
        self.service.load(&link, &addresses).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::payment::{GemPaymentAmount, GemPaymentLink, GemPaymentRequest};
    use primitives::{AssetId, Chain};

    #[test]
    fn test_request() {
        assert_eq!(
            payment_decode_url("solana:3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw?amount=0.42301").unwrap(),
            GemPayment::Request(GemPaymentRequest {
                address: "3u3ta6yXYgpheLGc2GVF3QkLHAUwBrvX71Eg8XXjJHGw".to_string(),
                amount: Some(GemPaymentAmount::ExactValue("0.42301".to_string())),
                memo: None,
                asset_id: Some(AssetId::from_chain(Chain::Solana)),
                references: None,
            })
        );
    }

    #[test]
    fn test_link() {
        const CONSTANT_K: &str = "https://www.constant-k.com/ck-txreq/?tok=MjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4";

        assert_eq!(
            payment_decode_url("solana:https%3A%2F%2Fapi.spherepay.co%2Fv1%2Fpublic%2FpaymentLink%2Fpay%2FpaymentLink_1").unwrap(),
            GemPayment::Link(GemPaymentLink::SolanaPay {
                url: "https://api.spherepay.co/v1/public/paymentLink/pay/paymentLink_1".to_string(),
            })
        );
        assert_eq!(
            payment_decode_url("solana:https%3A%2F%2Fwww.constant-k.com%2Fck-txreq%2F%3Ftok%3DMjYyfG9wZXJhdG9yfGFubnVhbHx8MTc4NzUyOTMxOXw3M2FiNDFhZmIwNTAxZWNjNjE2Y2E4NmIxZGE5N2FlOWZjM2Y1OGMzZWZhMGYxMjNiOGI4ZGYzZmU2YzQ3ZmM4").unwrap(),
            GemPayment::Link(GemPaymentLink::SolanaPay {
                url: CONSTANT_K.to_string(),
            })
        );
        assert!(payment_decode_url("https://pay.walletconnect.com/?pid=pay_123").is_err());
    }
}
