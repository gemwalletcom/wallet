use crate::{FiatWebhookRequest, error::FiatQuoteError, rsa_signature::verify_rsa_pss_signature};
use std::error::Error;

use super::client::PaybisClient;

const SIGNATURE_HEADER: &str = "x-request-signature";
const PAYBIS_WEBHOOK_PUBLIC_KEY: &str = include_str!("../../../testdata/paybis/paybis_webhook_public_key.pem");

impl PaybisClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let signature = request
            .header(SIGNATURE_HEADER)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Paybis webhook signature".to_string()))?;

        if verify_rsa_pss_signature(PAYBIS_WEBHOOK_PUBLIC_KEY, &request.raw_body, signature)? {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Paybis webhook signature".to_string()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{FiatWebhookRequest, providers::paybis::client::PaybisClient};

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = FiatWebhookRequest::mock_with_header(
            r#"{"event":"TRANSACTION_STATUS_CHANGED","data":{"transaction":{"invoice":"PB1"}}}"#,
            "x-request-signature",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert!(PaybisClient::mock().verify_webhook(&request).is_err());
    }

    #[test]
    fn test_verify_webhook_rejects_missing_signature() {
        let request = FiatWebhookRequest::mock(r#"{"event":"TRANSACTION_STATUS_CHANGED"}"#);

        assert!(PaybisClient::mock().verify_webhook(&request).is_err());
    }
}
