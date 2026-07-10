use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use std::{collections::HashMap, error::Error, fmt};

const SIGNATURE_HEADER: &str = "x-chatwoot-signature";
const TIMESTAMP_HEADER: &str = "x-chatwoot-timestamp";
const SIGNATURE_PREFIX: &str = "sha256=";
const MAX_SIGNATURE_AGE_SECONDS: i64 = 300;

pub struct ChatwootWebhookVerifier {
    secret: String,
}

#[derive(Debug)]
pub enum ChatwootWebhookError {
    InvalidRequest(String),
}

impl fmt::Display for ChatwootWebhookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(message) => write!(formatter, "{message}"),
        }
    }
}

impl Error for ChatwootWebhookError {}

impl ChatwootWebhookVerifier {
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    pub fn verify(&self, headers: &HashMap<String, String>, data: &str) -> Result<(), ChatwootWebhookError> {
        self.verify_at(headers, data, chrono::Utc::now().timestamp())
    }

    fn verify_at(&self, headers: &HashMap<String, String>, data: &str, now: i64) -> Result<(), ChatwootWebhookError> {
        let timestamp = headers
            .get(TIMESTAMP_HEADER)
            .ok_or_else(|| ChatwootWebhookError::InvalidRequest("Missing Chatwoot webhook timestamp".to_string()))?;
        let signed_at = timestamp
            .parse::<i64>()
            .map_err(|_| ChatwootWebhookError::InvalidRequest("Invalid Chatwoot webhook timestamp".to_string()))?;
        if now.abs_diff(signed_at) > MAX_SIGNATURE_AGE_SECONDS as u64 {
            return Err(ChatwootWebhookError::InvalidRequest("Expired Chatwoot webhook timestamp".to_string()));
        }
        let signature = headers
            .get(SIGNATURE_HEADER)
            .and_then(|signature| signature.strip_prefix(SIGNATURE_PREFIX))
            .and_then(|signature| hex::decode(signature).ok())
            .ok_or_else(|| ChatwootWebhookError::InvalidRequest("Invalid Chatwoot webhook signature".to_string()))?;

        let mut mac =
            Hmac::<Sha256>::new_from_slice(self.secret.as_bytes()).map_err(|_| ChatwootWebhookError::InvalidRequest("Invalid Chatwoot webhook signing key".to_string()))?;
        mac.update(format!("{timestamp}.{data}").as_bytes());
        mac.verify_slice(&signature)
            .map_err(|_| ChatwootWebhookError::InvalidRequest("Invalid Chatwoot webhook signature".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SECRET: &str = "test_chatwoot_webhook_secret";
    const TEST_NOW: i64 = 1_750_000_000;
    const TEST_DATA: &str = r#"{"event":"message_created"}"#;

    fn signed_headers(data: &str, timestamp: i64, secret: &str) -> HashMap<String, String> {
        let timestamp = timestamp.to_string();
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(format!("{timestamp}.{data}").as_bytes());
        HashMap::from([
            (TIMESTAMP_HEADER.to_string(), timestamp),
            (SIGNATURE_HEADER.to_string(), format!("{SIGNATURE_PREFIX}{}", hex::encode(mac.finalize().into_bytes()))),
        ])
    }

    #[test]
    fn test_verify() {
        let verifier = ChatwootWebhookVerifier::new(TEST_SECRET.to_string());
        let headers = signed_headers(TEST_DATA, TEST_NOW, TEST_SECRET);
        assert!(verifier.verify_at(&headers, TEST_DATA, TEST_NOW).is_ok());
        assert!(verifier.verify_at(&headers, r#"{"event":"conversation_created"}"#, TEST_NOW).is_err());
        assert!(ChatwootWebhookVerifier::new("wrong_secret".to_string()).verify_at(&headers, TEST_DATA, TEST_NOW).is_err());
        assert!(verifier.verify_at(&headers, TEST_DATA, TEST_NOW + MAX_SIGNATURE_AGE_SECONDS + 1).is_err());
        assert!(verifier.verify_at(&headers, TEST_DATA, TEST_NOW - MAX_SIGNATURE_AGE_SECONDS - 1).is_err());

        let mut missing_signature = headers.clone();
        missing_signature.remove(SIGNATURE_HEADER);
        assert!(verifier.verify_at(&missing_signature, TEST_DATA, TEST_NOW).is_err());

        let mut malformed_signature = headers.clone();
        malformed_signature.insert(SIGNATURE_HEADER.to_string(), "invalid".to_string());
        assert!(verifier.verify_at(&malformed_signature, TEST_DATA, TEST_NOW).is_err());

        let mut missing_timestamp = headers;
        missing_timestamp.remove(TIMESTAMP_HEADER);
        assert!(verifier.verify_at(&missing_timestamp, TEST_DATA, TEST_NOW).is_err());
    }
}
