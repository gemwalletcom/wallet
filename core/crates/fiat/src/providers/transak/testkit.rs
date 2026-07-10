use jsonwebtoken::{EncodingKey, Header, encode};
use serde_json::Value;

use crate::FiatWebhookRequest;

use super::client::TransakClient;

const TEST_ACCESS_TOKEN: &str = "test_access_token";

impl TransakClient {
    pub fn mock() -> Self {
        Self::mock_with_access_token(TEST_ACCESS_TOKEN)
    }

    pub fn mock_with_access_token(access_token: &str) -> Self {
        Self::new_with_access_token(access_token)
    }
}

impl FiatWebhookRequest {
    pub fn mock_transak_signed(claims: Value) -> Self {
        let jwt = encode(&Header::default(), &claims, &EncodingKey::from_secret(TEST_ACCESS_TOKEN.as_bytes())).unwrap();
        Self::from_value(serde_json::json!({ "data": jwt }))
    }
}
