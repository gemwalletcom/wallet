use crate::GemstoneError;
use primitives::{AssetId, Payment, PaymentAmount, PaymentLink, PaymentRequest, PaymentURLDecoder};

pub type GemPayment = Payment;
pub type GemPaymentRequest = PaymentRequest;
pub type GemPaymentLink = PaymentLink;
pub type GemPaymentAmount = PaymentAmount;

#[uniffi::remote(Enum)]
pub enum GemPayment {
    Request(GemPaymentRequest),
    Link(GemPaymentLink),
}

#[uniffi::remote(Enum)]
pub enum GemPaymentAmount {
    ExactValue(String),
    AtomicValue(String),
}

#[uniffi::remote(Record)]
pub struct GemPaymentRequest {
    pub address: String,
    pub amount: Option<GemPaymentAmount>,
    pub memo: Option<String>,
    pub asset_id: Option<AssetId>,
}

#[uniffi::remote(Enum)]
pub enum GemPaymentLink {
    SolanaPay(String),
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
        assert!(payment_decode_url("https://pay.walletconnect.com/?pid=pay_123").is_err());
    }
}
