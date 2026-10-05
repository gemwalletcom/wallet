use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use gem_tracing::info_with_fields;
use primitives::TransactionId;
use streamer::{StreamProducerQueue, SupportWebhookPayload};

use crate::support::ChatwootWebhookVerifier;

#[derive(Debug)]
pub enum SupportWebhookError {
    Rejected(String),
    Publish(Box<dyn Error + Send + Sync>),
}

impl fmt::Display for SupportWebhookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rejected(message) => write!(f, "{message}"),
            Self::Publish(error) => write!(f, "{error}"),
        }
    }
}

impl Error for SupportWebhookError {}

pub struct WebhooksClient {
    stream_producer: Arc<dyn StreamProducerQueue>,
    chatwoot_webhook_verifier: ChatwootWebhookVerifier,
}

impl WebhooksClient {
    pub fn new(stream_producer: Arc<dyn StreamProducerQueue>, chatwoot_webhook_verifier: ChatwootWebhookVerifier) -> Self {
        Self { stream_producer, chatwoot_webhook_verifier }
    }

    pub async fn publish_support_webhook(&self, raw_body: &str, headers: &HashMap<String, String>) -> Result<(), SupportWebhookError> {
        self.chatwoot_webhook_verifier.verify(headers, raw_body).map_err(|error| SupportWebhookError::Rejected(error.to_string()))?;
        let webhook_data = serde_json::from_str(raw_body).map_err(|_| SupportWebhookError::Rejected("Invalid webhook JSON".to_string()))?;
        self.stream_producer.publish_support_webhook(SupportWebhookPayload::new(webhook_data)).await.map_err(SupportWebhookError::Publish)?;
        Ok(())
    }

    pub async fn publish_broadcast_webhook(&self, payload: TransactionId) -> Result<(), Box<dyn Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        info_with_fields!("received broadcast webhook", transaction_id = transaction_id.as_str());
        self.stream_producer.publish_pending_transaction(payload).await?;
        info_with_fields!("published broadcast webhook", transaction_id = transaction_id.as_str());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use hmac::{Hmac, KeyInit, Mac};
    use primitives::Chain;
    use serde_json::json;
    use sha2::Sha256;
    use streamer::QueueName;

    use super::*;
    use crate::testkit::RecordingStreamProducer;

    const SECRET: &str = "test_chatwoot_webhook_secret";
    const BODY: &str = r#"{"event":"message_created"}"#;

    fn signed_headers(body: &str) -> HashMap<String, String> {
        let timestamp = Utc::now().timestamp();
        let mut mac = Hmac::<Sha256>::new_from_slice(SECRET.as_bytes()).unwrap();
        mac.update(format!("{timestamp}.{body}").as_bytes());
        HashMap::from([
            ("x-chatwoot-timestamp".to_string(), timestamp.to_string()),
            ("x-chatwoot-signature".to_string(), format!("sha256={}", hex::encode(mac.finalize().into_bytes()))),
        ])
    }

    #[tokio::test]
    async fn test_publish_support_webhook() {
        let producer = Arc::new(RecordingStreamProducer::new());
        let client = WebhooksClient::new(producer.clone(), ChatwootWebhookVerifier::new(SECRET.to_string()));

        client.publish_support_webhook(BODY, &signed_headers(BODY)).await.unwrap();
        assert!(matches!(client.publish_support_webhook(BODY, &HashMap::new()).await, Err(SupportWebhookError::Rejected(_))));

        assert_eq!(producer.published(), vec![(QueueName::SupportWebhooks, json!({"data": {"event": "message_created"}}))]);
    }

    #[tokio::test]
    async fn test_publish_support_webhook_failure() {
        let client = WebhooksClient::new(Arc::new(RecordingStreamProducer::failing("broker unavailable")), ChatwootWebhookVerifier::new(SECRET.to_string()));

        let result = client.publish_support_webhook(BODY, &signed_headers(BODY)).await;

        assert!(matches!(result, Err(SupportWebhookError::Publish(error)) if error.to_string() == "broker unavailable"));
    }

    #[tokio::test]
    async fn test_publish_broadcast_webhook() {
        let producer = Arc::new(RecordingStreamProducer::new());
        let client = WebhooksClient::new(producer.clone(), ChatwootWebhookVerifier::new(SECRET.to_string()));

        client.publish_broadcast_webhook(TransactionId::new(Chain::Ethereum, "0x123".to_string())).await.unwrap();

        assert_eq!(producer.published(), vec![(QueueName::StorePendingTransactions, json!("ethereum_0x123"))]);
    }
}
