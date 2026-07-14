use std::error::Error;

use crate::{FiatWebhookRequest, error::FiatQuoteError, hmac_signature::verify_hmac_signature_hex};

use super::client::FlashnetClient;

const SIGNATURE_HEADER: &str = "x-flashnet-signature";
const TIMESTAMP_HEADER: &str = "x-flashnet-timestamp";

impl FlashnetClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let signature = request
            .header(SIGNATURE_HEADER)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Flashnet webhook signature".to_string()))?;
        let timestamp = request
            .header(TIMESTAMP_HEADER)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Flashnet webhook timestamp".to_string()))?;
        let signed_payload = format!("{timestamp}.{}", request.raw_body);
        if verify_hmac_signature_hex(&self.webhook_secret_key, &signed_payload, signature) {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Flashnet webhook signature".to_string()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatWebhookRequest, providers::flashnet::client::FlashnetClient};

    #[test]
    fn test_verify_webhook() {
        let request = FiatWebhookRequest::mock_flashnet_signed(include_str!("../../../testdata/flashnet/webhook_completed.json"));

        FlashnetClient::mock().verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = FiatWebhookRequest::mock_flashnet_with_signature(
            include_str!("../../../testdata/flashnet/webhook_completed.json"),
            "1700000000000",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert!(FlashnetClient::mock().verify_webhook(&request).is_err());
    }
}
