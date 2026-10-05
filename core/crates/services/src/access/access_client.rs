use std::sync::Arc;

use primitives::WebhookKind;
use storage::{ApiClientResource, ApiClientScope, DatabaseError};

use super::repository::Repository;

pub struct AccessClient {
    repository: Arc<dyn Repository>,
}

impl AccessClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn is_api_client_allowed(&self, secret: &str, scope: ApiClientScope) -> Result<bool, DatabaseError> {
        self.repository.has_enabled_api_client(secret.to_string(), scope, ApiClientResource::Global).await
    }

    pub async fn is_webhook_sender_allowed(&self, secret: &str, kind: WebhookKind, sender: &str) -> Result<bool, DatabaseError> {
        self.repository
            .has_enabled_api_client(secret.to_string(), ApiClientScope::webhook(kind), ApiClientResource::WebhookSender(sender.to_string()))
            .await
    }
}
