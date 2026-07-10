use std::collections::HashMap;

use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};

use super::client::FlashnetClient;

const TEST_API_KEY: &str = "test_api_key";
const TEST_AFFILIATE_ID: &str = "test_affiliate";

impl FlashnetClient {
    pub fn mock() -> Self {
        Self::new(gem_client::reqwest_client(), String::new(), TEST_API_KEY.to_string(), TEST_AFFILIATE_ID.to_string())
    }
}

impl FiatWebhookRequest {
    pub fn mock_flashnet_signed(raw_body: &str) -> Self {
        let signature = generate_hmac_signature_hex(TEST_API_KEY, raw_body);
        Self::mock_flashnet_with_signature(raw_body, &signature)
    }

    pub fn mock_flashnet_with_signature(raw_body: &str, signature: &str) -> Self {
        Self::new(
            raw_body.to_string(),
            HashMap::from([("x-flashnet-signature".to_string(), signature.to_string())]),
            String::new(),
        )
        .unwrap()
    }
}
