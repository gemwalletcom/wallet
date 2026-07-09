use crate::{FiatWebhookRequest, error::FiatQuoteError};
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::error::Error;

use super::client::MercuryoClient;

const SIGNATURE_HEADER: &str = "x-signature";

type HmacSha256 = Hmac<Sha256>;

impl MercuryoClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        if self.webhook_secret_key.is_empty() {
            return Err(FiatQuoteError::InvalidRequest("Missing Mercuryo webhook signing key".to_string()).into());
        }

        let signature = request
            .header(SIGNATURE_HEADER)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Mercuryo webhook signature".to_string()))?;
        let signature = hex::decode(signature).map_err(|_| FiatQuoteError::InvalidRequest("Invalid Mercuryo webhook signature".to_string()))?;

        let mut mac = HmacSha256::new_from_slice(self.webhook_secret_key.as_bytes()).expect("HMAC can take key of any size");
        mac.update(request.raw_body.as_bytes());
        mac.verify_slice(&signature)
            .map_err(|_| FiatQuoteError::InvalidRequest("Invalid Mercuryo webhook signature".to_string()).into())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{FiatWebhookRequest, providers::mercuryo::client::MercuryoClient};

    const TEST_KEY: &str = "test_webhook_key";

    fn client(webhook_secret_key: &str) -> MercuryoClient {
        MercuryoClient::new(gem_client::reqwest_client(), String::new(), String::new(), webhook_secret_key.to_string())
    }

    fn request(body: &str, signature_header: &str) -> FiatWebhookRequest {
        FiatWebhookRequest::new(body.to_string(), HashMap::from([("x-signature".to_string(), signature_header.to_string())])).unwrap()
    }

    #[test]
    fn test_verify_webhook() {
        let body = r#"{"data":{"id":"tx_1"}}"#;
        let signature = "56d8e73de023d5375e7ae037b2cab2b15884002e8461f24cd3b6be2a33d6276d";
        let request = request(body, signature);

        client(TEST_KEY).verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = request(r#"{"data":{"id":"tx_1"}}"#, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");

        assert!(client(TEST_KEY).verify_webhook(&request).is_err());
    }

    #[test]
    fn test_verify_webhook_rejects_missing_key() {
        let request = request(r#"{"data":{"id":"tx_1"}}"#, "56d8e73de023d5375e7ae037b2cab2b15884002e8461f24cd3b6be2a33d6276d");

        assert!(client("").verify_webhook(&request).is_err());
    }
}
