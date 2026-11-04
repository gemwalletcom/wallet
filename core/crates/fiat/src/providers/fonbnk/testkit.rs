use gem_client::ReqwestClient;

use crate::FiatWebhookRequest;

use super::client::FonbnkClient;
use super::webhook::webhook_signature;

const TEST_WEBHOOK_SECRET: &str = "test_webhook_secret";

impl FonbnkClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            "test_client_id".to_string(),
            "dGVzdF9hcGlfc2VjcmV0".to_string(),
            "https://pay.fonbnk.com".to_string(),
            "test_widget_secret".to_string(),
            TEST_WEBHOOK_SECRET.to_string(),
        )
    }
}

impl FiatWebhookRequest {
    pub fn mock_fonbnk_signed(raw_body: &str) -> Self {
        Self::mock_with_header(raw_body, "x-signature", &hex::encode(webhook_signature(TEST_WEBHOOK_SECRET, raw_body)))
    }
}
