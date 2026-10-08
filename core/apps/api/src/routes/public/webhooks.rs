use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::{DefaultBodyLimit, State};
use axum::routing::post;
use fiat::FiatWebhookRequest;
use primitives::{TransactionId, WebhookKind};
use services::access::AccessClient;
use services::fiat::FiatClient;
use services::webhooks::{SupportWebhookError, WebhooksClient};

use crate::auth::webhook::{WebhookRequest, WebhookSecret};
use crate::error::ApiError;
use crate::request::{Path, WebhookKindParam};
use crate::response::ApiResponse;
use crate::state::AppState;

const MAX_WEBHOOK_BODY_BYTES: usize = 1024 * 1024;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/webhooks/{kind}/{sender}/{secret}", post(create_webhook))
        .route("/webhooks/{kind}/{sender}", post(create_webhook_with_header))
        .layer(DefaultBodyLimit::max(MAX_WEBHOOK_BODY_BYTES))
}

async fn authorize_webhook(access: &AccessClient, kind: WebhookKind, sender: &str, secret: &str) -> Result<(), ApiError> {
    let exists = access
        .is_webhook_sender_allowed(secret, kind, sender)
        .await
        .map_err(|error| ApiError::internal(format!("Failed to load webhook endpoint: {error}")))?;
    if !exists {
        return Err(ApiError::not_found("Webhook endpoint not found"));
    }
    Ok(())
}

async fn receive_webhook(
    kind: WebhookKind,
    sender: &str,
    secret: &str,
    access: &AccessClient,
    body: Bytes,
    webhook_request: WebhookRequest,
    fiat_client: &FiatClient,
    webhooks_client: &WebhooksClient,
) -> Result<ApiResponse<bool>, ApiError> {
    authorize_webhook(access, kind, sender, secret).await?;

    let raw_body = String::from_utf8(body.to_vec()).map_err(|_| ApiError::bad_request("Webhook body is not valid UTF-8"))?;
    match kind {
        WebhookKind::Transactions => {
            let payload: TransactionId = serde_json::from_str(&raw_body).map_err(|_| ApiError::bad_request("Invalid webhook JSON"))?;
            webhooks_client.publish_broadcast_webhook(payload).await?;
        }
        WebhookKind::Support => {
            webhooks_client.publish_support_webhook(&raw_body, &webhook_request.headers).await.map_err(|error| match error {
                SupportWebhookError::Rejected(message) => ApiError::bad_request(message),
                SupportWebhookError::Publish(error) => ApiError::from(error),
            })?;
        }
        WebhookKind::Fiat => {
            let request = FiatWebhookRequest::new(raw_body, webhook_request.headers, webhook_request.path).map_err(|_| ApiError::bad_request("Invalid webhook JSON"))?;
            fiat_client.publish_webhook(request, sender).await?;
        }
    }
    Ok(true.into())
}

async fn create_webhook(
    Path((kind, sender, secret)): Path<(WebhookKindParam, String, String)>,
    webhook_request: WebhookRequest,
    State(access): State<Arc<AccessClient>>,
    State(fiat_client): State<Arc<FiatClient>>,
    State(webhooks_client): State<Arc<WebhooksClient>>,
    body: Bytes,
) -> Result<ApiResponse<bool>, ApiError> {
    receive_webhook(kind.0, &sender, &secret, &access, body, webhook_request, &fiat_client, &webhooks_client).await
}

async fn create_webhook_with_header(
    Path((kind, sender)): Path<(WebhookKindParam, String)>,
    secret: WebhookSecret,
    webhook_request: WebhookRequest,
    State(access): State<Arc<AccessClient>>,
    State(fiat_client): State<Arc<FiatClient>>,
    State(webhooks_client): State<Arc<WebhooksClient>>,
    body: Bytes,
) -> Result<ApiResponse<bool>, ApiError> {
    receive_webhook(kind.0, &sender, &secret.0, &access, body, webhook_request, &fiat_client, &webhooks_client).await
}
