use crate::{FiatWebhookRequest, error::FiatQuoteError, hmac_signature::verify_hmac_signature_hex};
use std::error::Error;

use super::client::MoonPayClient;

const SIGNATURE_HEADER: &str = "moonpay-signature-v2";

impl MoonPayClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (timestamp, signature) = request
            .header(SIGNATURE_HEADER)
            .and_then(parse_signature_header)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Invalid MoonPay webhook signature header".to_string()))?;

        let signed_payload = format!("{}.{}", timestamp, request.raw_body);
        if verify_hmac_signature_hex(&self.webhook_secret_key, &signed_payload, signature) {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid MoonPay webhook signature".to_string()).into())
        }
    }
}

fn parse_signature_header(header: &str) -> Option<(&str, &str)> {
    let mut timestamp = None;
    let mut signature = None;

    for item in header.split(',') {
        let (key, value) = item.trim().split_once('=')?;
        match key {
            "t" => timestamp = Some(value),
            "s" => signature = Some(value),
            _ => {}
        }
    }

    Some((timestamp?, signature?))
}

#[cfg(test)]
mod tests {
    use crate::{FiatWebhookRequest, providers::moonpay::client::MoonPayClient};

    #[test]
    fn test_verify_webhook() {
        let body = r#"{"data":{"id":"tx_1"}}"#;
        let request = FiatWebhookRequest::mock_moonpay_signed(body);

        MoonPayClient::mock().verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = FiatWebhookRequest::mock_moonpay_with_signature(
            r#"{"data":{"id":"tx_1"}}"#,
            "t=1492774577,s=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert!(MoonPayClient::mock().verify_webhook(&request).is_err());
    }
}
