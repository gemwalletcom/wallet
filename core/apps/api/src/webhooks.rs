use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};
use gem_tracing::info_with_fields;
use primitives::{TransactionId, WebhookKind};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromParam, FromRequest, Outcome};
use rocket::{Request, State, post, serde::json::Json, tokio::sync::Mutex};
use std::str::FromStr;
use storage::{ApiClientResource, ApiClientScope, ApiClientsRepository, Database};
use streamer::{QueueName, StreamProducer, SupportWebhookPayload};

use crate::devices::FiatQuotesClient;
use crate::responders::{ApiError, ApiResponse};

pub struct WebhooksClient {
    stream_producer: StreamProducer,
}

impl WebhooksClient {
    pub fn new(stream_producer: StreamProducer) -> Self {
        Self { stream_producer }
    }

    pub async fn process_support_webhook(&self, webhook_data: serde_json::Value) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = SupportWebhookPayload::new(webhook_data);
        self.stream_producer.publish(QueueName::SupportWebhooks, &payload).await?;
        Ok(())
    }

    pub async fn process_broadcast_webhook(&self, payload: TransactionId) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transaction_id = payload.to_string();
        info_with_fields!("received broadcast webhook", transaction_id = transaction_id.as_str());
        self.stream_producer.publish(QueueName::StorePendingTransactions, &payload).await?;
        info_with_fields!("published broadcast webhook", transaction_id = transaction_id.as_str());
        Ok(())
    }
}

pub struct WebhookKindParam(WebhookKind);

impl<'r> FromParam<'r> for WebhookKindParam {
    type Error = &'r str;

    fn from_param(param: &'r str) -> Result<Self, Self::Error> {
        WebhookKind::from_str(param).map(Self).map_err(|_| param)
    }
}

pub struct WebhookSecret(String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebhookSecret {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let Some(auth_value) = req.headers().get_one(AUTHORIZATION_HEADER) else {
            return Error((Status::Unauthorized, "Missing Authorization header".to_string()));
        };

        match auth_value.strip_prefix(BEARER_PREFIX).filter(|secret| !secret.is_empty()) {
            Some(secret) => Success(Self(secret.to_string())),
            None => Error((Status::Unauthorized, "Invalid authorization format".to_string())),
        }
    }
}

fn authorize_webhook(database: &State<Database>, kind: WebhookKind, sender: &str, secret: &str) -> Result<(), ApiError> {
    database
        .api_clients()
        .and_then(|mut client| Ok(client.get_enabled_api_client(secret, ApiClientScope::webhook(kind), ApiClientResource::WebhookSender(sender.to_string()))?))
        .map_err(|_| ApiError::InternalServerError("Failed to load webhook endpoint".to_string()))?
        .ok_or_else(|| ApiError::NotFound("Webhook endpoint not found".to_string()))?;

    Ok(())
}

async fn process_webhook(
    kind: WebhookKindParam,
    sender: &str,
    secret: &str,
    database: &State<Database>,
    webhook_data: Json<serde_json::Value>,
    fiat_quotes_client: &State<Mutex<FiatQuotesClient>>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<ApiResponse<bool>, ApiError> {
    authorize_webhook(database, kind.0, sender, secret)?;

    let webhook_data = webhook_data.0;
    match kind.0 {
        WebhookKind::Transactions => {
            let payload: TransactionId = serde_json::from_value(webhook_data)?;
            webhooks_client.lock().await.process_broadcast_webhook(payload).await?;
        }
        WebhookKind::Support => {
            webhooks_client.lock().await.process_support_webhook(webhook_data).await?;
        }
        WebhookKind::Fiat => {
            fiat_quotes_client.lock().await.process_and_publish_webhook(sender, webhook_data).await?;
        }
    }
    Ok(true.into())
}

#[post("/webhooks/<kind>/<sender>/<secret>", data = "<webhook_data>")]
pub async fn create_webhook(
    kind: WebhookKindParam,
    sender: &str,
    secret: &str,
    database: &State<Database>,
    webhook_data: Json<serde_json::Value>,
    fiat_quotes_client: &State<Mutex<FiatQuotesClient>>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, secret, database, webhook_data, fiat_quotes_client, webhooks_client).await
}

#[post("/webhooks/<kind>/<sender>", data = "<webhook_data>")]
pub async fn create_webhook_with_header(
    kind: WebhookKindParam,
    sender: &str,
    secret: WebhookSecret,
    database: &State<Database>,
    webhook_data: Json<serde_json::Value>,
    fiat_quotes_client: &State<Mutex<FiatQuotesClient>>,
    webhooks_client: &State<Mutex<WebhooksClient>>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, &secret.0, database, webhook_data, fiat_quotes_client, webhooks_client).await
}
