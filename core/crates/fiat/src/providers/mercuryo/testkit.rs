use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};
use gem_client::ReqwestClient;

use super::client::MercuryoClient;

pub const TEST_WEBHOOK_SIGNING_KEY: &str = "test_webhook_key";

impl MercuryoClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            String::new(),
            String::new(),
            TEST_WEBHOOK_SIGNING_KEY.to_string(),
        )
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
