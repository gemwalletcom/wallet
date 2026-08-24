use crate::GemstoneError;
use crate::models::payment::GemPayment;
use primitives::PaymentURLDecoder;

#[uniffi::export]
pub fn payment_decode_url(string: &str) -> Result<GemPayment, GemstoneError> {
    Ok(PaymentURLDecoder::decode(string)?)
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
