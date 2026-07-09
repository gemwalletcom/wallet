use crate::{FiatWebhookRequest, error::FiatQuoteError, hmac_signature::verify_hmac_signature_hex};
use std::error::Error;

use super::client::MercuryoClient;

const SIGNATURE_HEADER: &str = "x-signature";

impl MercuryoClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.webhook_secret_key.is_empty() {
            return Err(FiatQuoteError::InvalidRequest("Missing Mercuryo webhook signing key".to_string()).into());
        }

        let signature = request
            .header(SIGNATURE_HEADER)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Mercuryo webhook signature".to_string()))?;
        if verify_hmac_signature_hex(&self.webhook_secret_key, &request.raw_body, signature) {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Mercuryo webhook signature".to_string()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatWebhookRequest, providers::mercuryo::client::MercuryoClient};

    #[test]
    fn test_verify_webhook() {
        let body = r#"{"data":{"id":"tx_1"}}"#;
        let request = FiatWebhookRequest::mock_mercuryo_signed(body);

        MercuryoClient::mock().verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = FiatWebhookRequest::mock_mercuryo_with_signature(r#"{"data":{"id":"tx_1"}}"#, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        assert!(MercuryoClient::mock().verify_webhook(&request).is_err());
    }

    #[test]
    fn test_verify_webhook_rejects_missing_key() {
        let request = FiatWebhookRequest::mock_mercuryo_signed(r#"{"data":{"id":"tx_1"}}"#);

        assert!(MercuryoClient::mock_with_webhook_secret_key("").verify_webhook(&request).is_err());
    }
}
