use gem_client::ReqwestClient;
use gem_encoding::{decode_base64, encode_base64_url};
use jsonwebtoken::crypto::sign;
use jsonwebtoken::{Algorithm, EncodingKey};

use crate::FiatWebhookRequest;

use super::client::TopperClient;
use super::model::{TopperCountryHint, TopperOnRamp, TopperRecipient, TopperRecipientEditMode, TopperSource};

pub const TEST_WIDGET_ID: &str = "d259f6ac-3e8d-46eb-9e6c-c92e138b7660";
pub const TEST_KEY_ID: &str = "c084a85d-c486-4035-9c60-8cec81d8b8f5";
pub const TEST_SIGNING_KEY: &str = "MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgmdAQEjZBkxtoVeque2wXqebfo1HY0/C2uGApqeKEaX+hRANCAAR3GTRYoqE7IVtTR5kPSmWA4HyX2sz5IPk0OCcam+D32pyDHmxy8sorKn0CpG/mwqenVTcFag0dxSwOWVlGf8kP";
pub const TEST_SIGNING_PUBLIC_KEY: &str = r#"{"alg":"ES256","kty":"EC","crv":"P-256","x":"dxk0WKKhOyFbU0eZD0plgOB8l9rM-SD5NDgnGpvg99o","y":"nIMebHLyyisqfQKkb-bCp6dVNwVqDR3FLA5ZWUZ_yQ8"}"#;

impl TopperClient {
    pub fn mock() -> Self {
        Self::new(
            ReqwestClient::new(String::new(), gem_client::reqwest_client()),
            "https://app.topperpay.com".to_string(),
            TEST_WIDGET_ID.to_string(),
            TEST_KEY_ID.to_string(),
            TEST_SIGNING_KEY.to_string(),
            TEST_SIGNING_PUBLIC_KEY.to_string(),
        )
    }
}

impl TopperOnRamp {
    pub fn mock() -> Self {
        Self {
            source: TopperSource {
                amount: "100".to_string(),
                asset: "USD".to_string(),
            },
            target: TopperRecipient {
                asset: "ETH".to_string(),
                network: "ethereum".to_string(),
                address: Some("0xb794f5ea0ba39494ce839613fffba74279579268".to_string()),
                recipient_edit_mode: TopperRecipientEditMode::NotEditable,
            },
            simulation: TopperCountryHint { country: "US".to_string() },
        }
    }
}

impl FiatWebhookRequest {
    pub fn mock_topper_signed(raw_body: &str) -> Self {
        let header = encode_base64_url(br#"{"alg":"ES256"}"#);
        let message = format!("{header}.{}", encode_base64_url(raw_body.as_bytes()));
        let key = EncodingKey::from_ec_der(&decode_base64(TEST_SIGNING_KEY).unwrap());
        let signature = sign(message.as_bytes(), &key, Algorithm::ES256).unwrap();
        Self::mock_with_header(raw_body, "x-topper-jws-signature", &format!("{header}..{signature}"))
    }
}
