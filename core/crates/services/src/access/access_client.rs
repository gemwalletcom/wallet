use primitives::WebhookKind;
use storage::{ApiClientResource, ApiClientScope, ApiClientsRepository, Database, DatabaseError};

pub struct AccessClient {
    database: Database,
}

impl AccessClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn is_api_client_allowed(&self, secret: &str, scope: ApiClientScope) -> Result<bool, DatabaseError> {
        let secret = secret.to_string();
        self.database.run(move |client| client.has_enabled_api_client(&secret, scope, ApiClientResource::Global)).await
    }

    pub async fn is_webhook_sender_allowed(&self, secret: &str, kind: WebhookKind, sender: &str) -> Result<bool, DatabaseError> {
        let secret = secret.to_string();
        let resource = ApiClientResource::WebhookSender(sender.to_string());
        self.database.run(move |client| client.has_enabled_api_client(&secret, ApiClientScope::webhook(kind), resource)).await
    }
}
