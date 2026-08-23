use std::collections::HashMap;

use gem_client::ReqwestClient;

use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};

use super::client::FlashnetClient;

const TEST_API_KEY: &str = "test_api_key";
const TEST_WEBHOOK_SIGNING_KEY: &str = "test_webhook_key";
const TEST_AFFILIATE_ID: &str = "test_affiliate";

impl FlashnetClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            TEST_API_KEY.to_string(),
            TEST_AFFILIATE_ID.to_string(),
            TEST_WEBHOOK_SIGNING_KEY.to_string(),
        )
    }
}

impl FiatWebhookRequest {
    pub fn mock_flashnet_signed(raw_body: &str) -> Self {
        let timestamp = "1700000000000";
        let signed_payload = format!("{timestamp}.{raw_body}");
        let signature = generate_hmac_signature_hex(TEST_WEBHOOK_SIGNING_KEY, &signed_payload);
        Self::mock_flashnet_with_signature(raw_body, timestamp, &signature)
    }

    pub fn mock_flashnet_with_signature(raw_body: &str, timestamp: &str, signature: &str) -> Self {
        Self::new(
            raw_body.to_string(),
            HashMap::from([
                ("x-flashnet-signature".to_string(), signature.to_string()),
                ("x-flashnet-timestamp".to_string(), timestamp.to_string()),
            ]),
            String::new(),
        )
        .unwrap()
    }
}
