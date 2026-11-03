use std::error::Error;

use gem_crypto::compare::constant_time_eq;
use sha2::{Digest, Sha256};

use crate::{FiatWebhookRequest, error::FiatQuoteError};

use super::client::FonbnkClient;

const SIGNATURE_HEADER: &str = "x-signature";

impl FonbnkClient {
    pub(super) fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let signature = request.header(SIGNATURE_HEADER).ok_or_else(|| FiatQuoteError::InvalidRequest("Missing Fonbnk webhook signature".to_string()))?;
        let signature = hex::decode(signature).map_err(|_| FiatQuoteError::InvalidRequest("Invalid Fonbnk webhook signature".to_string()))?;
        if constant_time_eq(&webhook_signature(&self.webhook_secret, &request.raw_body), &signature) {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Fonbnk webhook signature".to_string()).into())
        }
    }
}

pub(super) fn webhook_signature(secret: &str, raw_body: &str) -> Vec<u8> {
    let secret_hash = hex::encode(Sha256::digest(secret.as_bytes()));
    Sha256::new().chain_update(raw_body.as_bytes()).chain_update(secret_hash.as_bytes()).finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    const WEBHOOK: &str = include_str!("../../../testdata/fonbnk/webhook_on_ramp.json");

    #[test]
    fn test_webhook_signature() {
        assert_eq!(hex::encode(webhook_signature("test_webhook_secret", WEBHOOK)), "4cf8e74a3c4f45867d19b34591e2d8554629b80aa84edeb793ff68c038480213");
    }

    #[test]
    fn test_verify_webhook() {
        let client = FonbnkClient::mock();

        assert!(client.verify_webhook(&FiatWebhookRequest::mock_fonbnk_signed(WEBHOOK)).is_ok());
        assert!(client.verify_webhook(&FiatWebhookRequest::mock(WEBHOOK)).is_err());
        assert!(client.verify_webhook(&FiatWebhookRequest::mock_with_header(WEBHOOK, SIGNATURE_HEADER, &"0".repeat(64))).is_err());
    }
}
