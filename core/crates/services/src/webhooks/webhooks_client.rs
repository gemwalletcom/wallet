use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use gem_tracing::info_with_fields;
use primitives::TransactionId;
use streamer::{QueueName, StreamProducer, SupportWebhookPayload};

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
    stream_producer: StreamProducer,
    chatwoot_webhook_verifier: ChatwootWebhookVerifier,
}

impl WebhooksClient {
    pub fn new(stream_producer: StreamProducer, support_webhook_secret: String) -> Self {
        Self {
            stream_producer,
            chatwoot_webhook_verifier: ChatwootWebhookVerifier::new(support_webhook_secret),
        }
    }

    pub async fn process_support_webhook(&self, raw_body: &str, headers: &HashMap<String, String>) -> Result<(), SupportWebhookError> {
        self.chatwoot_webhook_verifier.verify(headers, raw_body).map_err(|error| SupportWebhookError::Rejected(error.to_string()))?;
        let webhook_data = serde_json::from_str(raw_body).map_err(|_| SupportWebhookError::Rejected("Invalid webhook JSON".to_string()))?;
        self.stream_producer
            .publish(QueueName::SupportWebhooks, &SupportWebhookPayload::new(webhook_data))
            .await
            .map_err(SupportWebhookError::Publish)?;
        Ok(())
    }

    pub async fn process_broadcast_webhook(&self, payload: TransactionId) -> Result<(), Box<dyn Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        info_with_fields!("received broadcast webhook", transaction_id = transaction_id.as_str());
        self.stream_producer.publish(QueueName::StorePendingTransactions, &payload).await?;
        info_with_fields!("published broadcast webhook", transaction_id = transaction_id.as_str());
        Ok(())
    }
}
