use crate::{FiatWebhookRequest, error::FiatQuoteError};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::error::Error;

use super::client::MoonPayClient;

const SIGNATURE_HEADER: &str = "moonpay-signature-v2";

type HmacSha256 = Hmac<Sha256>;

impl MoonPayClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.webhook_secret_key.is_empty() {
            return Err(FiatQuoteError::InvalidRequest("Missing MoonPay webhook signing key".to_string()).into());
        }

        let (timestamp, signature) = request
            .header(SIGNATURE_HEADER)
            .and_then(parse_signature_header)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Invalid MoonPay webhook signature header".to_string()))?;

        let signature = hex::decode(signature).map_err(|_| FiatQuoteError::InvalidRequest("Invalid MoonPay webhook signature".to_string()))?;
        let signed_payload = format!("{}.{}", timestamp, request.raw_body);
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret_key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(signed_payload.as_bytes());
        mac.verify_slice(&signature)
            .map_err(|_| FiatQuoteError::InvalidRequest("Invalid MoonPay webhook signature".to_string()).into())
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
    use std::collections::HashMap;

    use crate::{FiatWebhookRequest, providers::moonpay::client::MoonPayClient};

    const TEST_KEY: &str = "test_webhook_key";

    fn client(webhook_secret_key: &str) -> MoonPayClient {
        MoonPayClient::new(gem_client::reqwest_client(), String::new(), String::new(), webhook_secret_key.to_string())
    }

    fn request(body: &str, signature_header: &str) -> FiatWebhookRequest {
        FiatWebhookRequest::new(body.to_string(), HashMap::from([("moonpay-signature-v2".to_string(), signature_header.to_string())])).unwrap()
    }

    #[test]
    fn test_verify_webhook() {
        let body = r#"{"data":{"id":"tx_1"}}"#;
        let signature = "9eebbd10f8d103400831fcd0972baac1edf8f6db7666299b6459b805be23984b";
        let request = request(body, &format!("t=1492774577,s={signature}"));

        client(TEST_KEY).verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = request(
            r#"{"data":{"id":"tx_1"}}"#,
            "t=1492774577,s=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        );

        assert!(client(TEST_KEY).verify_webhook(&request).is_err());
    }

    #[test]
    fn test_verify_webhook_rejects_missing_key() {
        let request = request(
            r#"{"data":{"id":"tx_1"}}"#,
            "t=1492774577,s=9eebbd10f8d103400831fcd0972baac1edf8f6db7666299b6459b805be23984b",
        );

        assert!(client("").verify_webhook(&request).is_err());
    }
}
