use fiat::FiatWebhookRequest;
use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};
use primitives::{TransactionId, WebhookKind};
use rocket::data::{Data, ToByteUnit};
use rocket::http::Status;
use rocket::outcome::Outcome::{Error, Success};
use rocket::request::{FromParam, FromRequest, Outcome};
use rocket::{Request, State, post};
use services::access::AccessClient;
use services::fiat::FiatClient;
use services::webhooks::{SupportWebhookError, WebhooksClient};
use std::{collections::HashMap, str::FromStr};

use crate::responders::{ApiError, ApiResponse};

const MAX_WEBHOOK_BODY_BYTES: u64 = 1024 * 1024;

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
        let headers = req.headers().iter().map(|header| (header.name().as_str().to_ascii_lowercase(), header.value().to_string())).collect();
        Success(Self {
            headers,
            path: req.uri().path().as_str().to_string(),
        })
    }
}

async fn authorize_webhook(access: &AccessClient, kind: WebhookKind, sender: &str, secret: &str) -> Result<(), ApiError> {
    let exists = access
        .is_webhook_sender_allowed(secret, kind, sender)
        .await
        .map_err(|error| ApiError::Internal(format!("Failed to load webhook endpoint: {error}")))?;
    if !exists {
        return Err(ApiError::NotFound("Webhook endpoint not found".to_string()));
    }
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
    access: &State<AccessClient>,
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_client: &State<FiatClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    authorize_webhook(access, kind.0, sender, secret).await?;

    let raw_body = read_webhook_body(webhook_data).await?;
    match kind.0 {
        WebhookKind::Transactions => {
            let payload: TransactionId = serde_json::from_str(&raw_body).map_err(|_| ApiError::BadRequest("Invalid webhook JSON".to_string()))?;
            webhooks_client.process_broadcast_webhook(payload).await?;
        }
        WebhookKind::Support => {
            webhooks_client.process_support_webhook(&raw_body, &webhook_request.headers).await.map_err(|error| match error {
                SupportWebhookError::Rejected(message) => ApiError::BadRequest(message),
                SupportWebhookError::Publish(error) => ApiError::from(error),
            })?;
        }
        WebhookKind::Fiat => {
            let request = FiatWebhookRequest::new(raw_body, webhook_request.headers, webhook_request.path).map_err(|_| ApiError::BadRequest("Invalid webhook JSON".to_string()))?;
            fiat_client.process_and_publish_webhook(request, sender).await?;
        }
    }
    Ok(true.into())
}

#[post("/webhooks/<kind>/<sender>/<secret>", data = "<webhook_data>")]
pub async fn create_webhook(
    kind: WebhookKindParam,
    sender: &str,
    secret: &str,
    access: &State<AccessClient>,
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_client: &State<FiatClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, secret, access, webhook_data, webhook_request, fiat_client, webhooks_client).await
}

#[post("/webhooks/<kind>/<sender>", data = "<webhook_data>")]
pub async fn create_webhook_with_header(
    kind: WebhookKindParam,
    sender: &str,
    secret: WebhookSecret,
    access: &State<AccessClient>,
    webhook_data: Data<'_>,
    webhook_request: WebhookRequest,
    fiat_client: &State<FiatClient>,
    webhooks_client: &State<WebhooksClient>,
) -> Result<ApiResponse<bool>, ApiError> {
    process_webhook(kind, sender, &secret.0, access, webhook_data, webhook_request, fiat_client, webhooks_client).await
}
