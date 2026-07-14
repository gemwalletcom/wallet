use std::error::Error;

use crate::{FiatWebhookRequest, error::FiatQuoteError, hmac_signature::verify_hmac_signature_hex};

use super::client::BanxaClient;

const AUTHORIZATION_HEADER: &str = "authorization";
const BEARER_PREFIX: &str = "Bearer ";

impl BanxaClient {
    pub fn verify_webhook(&self, request: &FiatWebhookRequest) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (_, signature, nonce) = request
            .header(AUTHORIZATION_HEADER)
            .and_then(parse_authorization_header)
            .ok_or_else(|| FiatQuoteError::InvalidRequest("Invalid Banxa webhook authorization header".to_string()))?;

        let message = format!("POST\n{}\n{nonce}\n{}", request.path, request.raw_body);
        if verify_hmac_signature_hex(&self.webhook_secret_key, &message, signature) {
            Ok(())
        } else {
            Err(FiatQuoteError::InvalidRequest("Invalid Banxa webhook signature".to_string()).into())
        }
    }
}

fn parse_authorization_header(header: &str) -> Option<(&str, &str, &str)> {
    let header = header.strip_prefix(BEARER_PREFIX)?;
    let mut parts = header.splitn(3, ':');
    let api_key = parts.next().filter(|value| !value.is_empty())?;
    let signature = parts.next().filter(|value| !value.is_empty())?;
    let nonce = parts.next().filter(|value| !value.is_empty())?;
    Some((api_key, signature, nonce))
}

#[cfg(test)]
mod tests {
    use crate::{FiatWebhookRequest, providers::banxa::client::BanxaClient};

    #[test]
    fn test_verify_webhook() {
        let request = FiatWebhookRequest::mock_banxa_signed(include_str!("../../../testdata/banxa/webhook_order_complete.json"));

        BanxaClient::mock().verify_webhook(&request).unwrap();
    }

    #[test]
    fn test_verify_webhook_rejects_invalid_signature() {
        let request = FiatWebhookRequest::mock_banxa_with_authorization(
            include_str!("../../../testdata/banxa/webhook_order_complete.json"),
            "Bearer test_merchant_key:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:1700000000",
        );

        assert!(BanxaClient::mock().verify_webhook(&request).is_err());
    }
}
