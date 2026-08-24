use std::collections::HashMap;

use crate::{FiatWebhookRequest, hmac_signature::generate_hmac_signature_hex};
use gem_client::ReqwestClient;

use super::client::BanxaClient;

const TEST_PARTNER: &str = "test_partner";
const TEST_API_KEY: &str = "test_secret_key";
const TEST_WEBHOOK_SECRET_KEY: &str = "test_webhook_secret_key";
const TEST_WEBHOOK_PATH: &str = "/v1/webhooks/fiat/banxa/test";

impl BanxaClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            String::new(),
            TEST_PARTNER.to_string(),
            TEST_API_KEY.to_string(),
            TEST_WEBHOOK_SECRET_KEY.to_string(),
        )
    }
}

impl FiatWebhookRequest {
    pub fn mock_banxa_signed(raw_body: &str) -> Self {
        let nonce = "1700000000";
        let message = format!("POST\n{TEST_WEBHOOK_PATH}\n{nonce}\n{raw_body}");
        let signature = generate_hmac_signature_hex(TEST_WEBHOOK_SECRET_KEY, &message);
        Self::mock_banxa_with_authorization(raw_body, &format!("Bearer {TEST_API_KEY}:{signature}:{nonce}"))
    }

    pub fn mock_banxa_with_authorization(raw_body: &str, authorization: &str) -> Self {
        Self::new(
            raw_body.to_string(),
            HashMap::from([("authorization".to_string(), authorization.to_string())]),
            TEST_WEBHOOK_PATH.to_string(),
        )
        .unwrap()
    }
}
