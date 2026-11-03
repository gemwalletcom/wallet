use std::error::Error;

use gem_encoding::encode_base64_url;
use jsonwebtoken::crypto::verify;
use jsonwebtoken::jwk::Jwk;
use jsonwebtoken::{Algorithm, DecodingKey};

use crate::{FiatWebhookRequest, error::FiatQuoteError};

use super::client::TopperClient;

const SIGNATURE_HEADER: &str = "x-topper-jws-signature";

impl TopperClient {
    pub(super) fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let signature = request.header(SIGNATURE_HEADER).ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Topper webhook signature".to_string()))?;
        if verify_signature(&self.webhook_key, signature, &request.raw_body)? {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Topper webhook signature".to_string()).into())
        }
    }
}

fn verify_signature(webhook_key: &str, signature: &str, raw_body: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let Some((header, signature)) = signature.split_once("..") else {
        return Ok(false);
    };
    let key = DecodingKey::from_jwk(&serde_json::from_str::<Jwk>(webhook_key)?)?;
    let message = format!("{header}.{}", encode_base64_url(raw_body.as_bytes()));
    Ok(verify(signature, message.as_bytes(), &key, Algorithm::ES256)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOPPER_DOCS_WEBHOOK_KEY: &str = r#"{"x":"7-INQ150R-MCWlj5X_wyGLRIRYAA-o8NakJiUq7gOGg","y":"dM-GsyJvdDOuALE3l-U9lPL8V3gY_5BPjLH539yTdKU","alg":"ES256","crv":"P-256","kid":"15a5142e-c20f-466e-8132-234dbdae97e7","kty":"EC"}"#;
    const TOPPER_DOCS_SIGNATURE: &str = "eyJhbGciOiJFUzI1NiJ9..2H0Ypm5sVzuSpgyZySdAJan05lYxctqhmO8btghFQQzkisvSlNvNWzQ1kqTPXTLP_dR4zQZrTsSsShAK51I4EQ";
    const WEBHOOK: &str = include_str!("../../../testdata/topper/webhook_order_completed.json");

    #[test]
    fn test_verify_signature() {
        assert!(verify_signature(TOPPER_DOCS_WEBHOOK_KEY, TOPPER_DOCS_SIGNATURE, r#"{"foo":"bar"}"#).unwrap());
        assert!(!verify_signature(TOPPER_DOCS_WEBHOOK_KEY, TOPPER_DOCS_SIGNATURE, r#"{"foo":"baz"}"#).unwrap());
        assert!(!verify_signature(TOPPER_DOCS_WEBHOOK_KEY, "eyJhbGciOiJFUzI1NiJ9", r#"{"foo":"bar"}"#).unwrap());
    }

    #[test]
    fn test_verify_webhook() {
        let client = TopperClient::mock();

        assert!(client.verify_webhook(&FiatWebhookRequest::mock_topper_signed(WEBHOOK)).is_ok());
        assert!(client.verify_webhook(&FiatWebhookRequest::mock(WEBHOOK)).is_err());
        assert!(client.verify_webhook(&FiatWebhookRequest::mock_with_header(WEBHOOK, SIGNATURE_HEADER, TOPPER_DOCS_SIGNATURE)).is_err());
    }
}
