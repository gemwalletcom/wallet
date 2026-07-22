use fiat::FiatWebhookRequest;
use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};
use gem_tracing::info_with_fields;
use primitives::{TransactionId, WebhookKind};
use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromParam, FromRequest, Outcome};
use rocket::{Request, State, post};
use std::{collections::HashMap, str::FromStr};
use storage::{ApiClientResource, ApiClientScope, ApiClientsRepository, Database};
use streamer::{QueueName, StreamProducer, SupportWebhookPayload};
use support::ChatwootWebhookVerifier;

use crate::devices::FiatQuotesClient;
use crate::responders::{ApiError, ApiResponse};

const MAX_WEBHOOK_BODY_BYTES: u64 = 1024 * 1024;

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

    pub async fn process_support_webhook(&self, raw_body: &str, headers: &HashMap<String, String>) -> Result<(), ApiError> {
        self.chatwoot_webhook_verifier
            .verify(headers, raw_body)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        let webhook_data = serde_json::from_str(raw_body).map_err(|_| ApiError::BadRequest("Invalid webhook JSON".to_string()))?;
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
pub struct WebhookRequest {
    headers: HashMap<String, String>,
    path: String,
}

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

#[rocket::async_trait]
impl<'r> FromRequest<'r> for WebhookRequest {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, String> {
        let headers = req
            .headers()
            .iter()
            .map(|header| (header.name().as_str().to_ascii_lowercase(), header.value().to_string()))
            .collect();
        Success(Self {
            headers,
            path: req.uri().path().as_str().to_string(),
        })
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

async fn read_webhook_body(webhook_data: Data<'_>) -> Result<String, ApiError> {
    let bytes = webhook_data
        .open(MAX_WEBHOOK_BODY_BYTES.bytes())
        .into_bytes()
        .await
        .map_err(|_| ApiError::BadRequest("Failed to read webhook body".to_string()))?;

    if !bytes.is_complete() {
        return Err(ApiError::BadRequest("Webhook body too large".to_string()));
    }

    String::from_utf8(bytes.into_inner()).map_err(|_| ApiError::BadRequest("Webhook body is not valid UTF-8".to_string()))
}

async fn process_webhook(
    kind: WebhookKindParam,
    sender: &str,
    secret: &str,
    database: &State<Database>,
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_quotes_client: &State<FiatQuotesClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    authorize_webhook(database, kind.0, sender, secret)?;

    let raw_body = read_webhook_body(webhook_data).await?;
    match kind.0 {
        WebhookKind::Transactions => {
            let payload: TransactionId = serde_json::from_str(&raw_body).map_err(|_| ApiError::BadRequest("Invalid webhook JSON".to_string()))?;
            webhooks_client.process_broadcast_webhook(payload).await?;
        }
        WebhookKind::Support => {
            webhooks_client.process_support_webhook(&raw_body, &webhook_request.headers).await?;
        }
        WebhookKind::Fiat => {
            let request = FiatWebhookRequest::new(raw_body, webhook_request.headers, webhook_request.path).map_err(|_| ApiError::BadRequest("Invalid webhook JSON".to_string()))?;
            fiat_quotes_client.process_and_publish_webhook(request, sender).await?;
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
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_quotes_client: &State<FiatQuotesClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, secret, database, webhook_data, webhook_request, fiat_quotes_client, webhooks_client).await
}

#[post("/webhooks/<kind>/<sender>", data = "<webhook_data>")]
pub async fn create_webhook_with_header(
    kind: WebhookKindParam,
    sender: &str,
    secret: WebhookSecret,
    database: &State<Database>,
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_quotes_client: &State<FiatQuotesClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, &secret.0, database, webhook_data, webhook_request, fiat_quotes_client, webhooks_client).await
}
