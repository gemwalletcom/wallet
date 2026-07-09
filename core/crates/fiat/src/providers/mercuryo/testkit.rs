use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};

use super::client::MercuryoClient;

pub const TEST_WEBHOOK_SIGNING_KEY: &str = "test_webhook_key";

impl MercuryoClient {
    pub fn mock() -> Self {
        Self::mock_with_webhook_secret_key(TEST_WEBHOOK_SIGNING_KEY)
    }

    pub fn mock_with_webhook_secret_key(webhook_secret_key: &str) -> Self {
        Self::new(gem_client::reqwest_client(), String::new(), String::new(), webhook_secret_key.to_string())
    }
}

impl FiatWebhookRequest {
    pub fn mock_mercuryo_signed(raw_body: &str) -> Self {
        let signature = generate_hmac_signature_hex(TEST_WEBHOOK_SIGNING_KEY, raw_body);
        Self::mock_mercuryo_with_signature(raw_body, &signature)
    }

    pub fn mock_mercuryo_with_signature(raw_body: &str, signature: &str) -> Self {
        Self::mock_with_header(raw_body, "x-signature", signature)
    }
}
