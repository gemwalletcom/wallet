use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, Bytes};
use axum::extract::{OriginalUri, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use gem_auth::{AUTHORIZATION_HEADER, device_auth_message, device_body_hash, parse_device_auth, verify_device_signature};
use gem_tracing::error_fields;
use http::{Method, Uri};
use http_body_util::LengthLimitError;
use primitives::hex::encode_with_0x;
use services::auth::AuthClient;

use super::{DEVICE_ID_LENGTH, DeviceAuthConfig, DeviceError};
use crate::error::ApiError;

#[derive(Clone, Copy, PartialEq)]
pub enum SignedPath {
    PathOnly,
    PathAndQuery,
}

#[derive(Clone)]
pub struct DeviceAuth {
    pub config: Arc<DeviceAuthConfig>,
    pub limit: usize,
    pub scheme: SignedPath,
    pub replay: Option<Arc<AuthClient>>,
}

#[derive(Clone)]
pub struct VerifiedRequest {
    pub device_id: String,
    pub wallet_id: Option<String>,
    pub body: Bytes,
}

pub async fn device_auth(State(auth): State<DeviceAuth>, OriginalUri(uri): OriginalUri, request: Request, next: Next) -> Response {
    match verify(&auth, &uri, request).await {
        Ok(request) => next.run(request).await,
        Err(error) => error.into_response(),
    }
}

async fn verify(auth: &DeviceAuth, uri: &Uri, request: Request) -> Result<Request, ApiError> {
    let (mut parts, body) = request.into_parts();
    let header = parts
        .headers
        .get(AUTHORIZATION_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized(DeviceError::MissingHeader(AUTHORIZATION_HEADER).to_string()))?;
    let payload = parse_device_auth(header).ok_or_else(|| ApiError::Unauthorized(DeviceError::InvalidAuthorizationFormat.to_string()))?;
    let timestamp_ms: u64 = payload.timestamp.parse().map_err(|_| ApiError::Unauthorized(DeviceError::InvalidTimestamp.to_string()))?;
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| ApiError::Unauthorized(DeviceError::InvalidTimestamp.to_string()))?.as_millis() as u64;
    if now_ms.abs_diff(timestamp_ms) > auth.config.tolerance.as_millis() as u64 {
        return Err(ApiError::Unauthorized(DeviceError::TimestampExpired.to_string()));
    }
    let signed_path = match auth.scheme {
        SignedPath::PathOnly => uri.path(),
        SignedPath::PathAndQuery => uri.path_and_query().map(http::uri::PathAndQuery::as_str).unwrap_or("/"),
    };
    let wallet_id = payload.wallet_id.as_deref().unwrap_or("");
    let message = device_auth_message(&payload.timestamp, parts.method.as_str(), signed_path, wallet_id, &payload.body_hash);
    if !verify_device_signature(&payload.device_id, &message, &payload.signature) {
        return Err(ApiError::Unauthorized(DeviceError::InvalidSignature.to_string()));
    }
    if payload.device_id.len() != DEVICE_ID_LENGTH {
        return Err(ApiError::Unauthorized(DeviceError::InvalidDeviceId.message(Some(&payload.device_id), None)));
    }
    let body = match axum::body::to_bytes(body, auth.limit).await {
        Ok(body) => body,
        Err(error) => {
            if error.into_inner().is::<LengthLimitError>() {
                return Err(ApiError::PayloadTooLarge("Request body too large".to_string()));
            }
            return Err(ApiError::BadRequest("Failed to read body".to_string()));
        }
    };
    if device_body_hash(&body) != payload.body_hash {
        return Err(ApiError::BadRequest("Body hash mismatch".to_string()));
    }
    if let Some(replay) = auth.replay.as_ref().filter(|_| is_mutation(&parts.method)) {
        match replay.remember_request_signature(&encode_with_0x(&payload.signature)).await {
            Ok(true) => {}
            Ok(false) => return Err(ApiError::Unauthorized(DeviceError::ReplayedRequest.message(Some(&payload.device_id), None))),
            Err(error) => error_fields!("replay check unavailable", error = error.to_string(), device_id = payload.device_id.as_str()),
        }
    }
    parts.extensions.insert(VerifiedRequest {
        device_id: payload.device_id,
        wallet_id: payload.wallet_id,
        body: body.clone(),
    });
    Ok(Request::from_parts(parts, Body::from(body)))
}

fn is_mutation(method: &Method) -> bool {
    matches!(*method, Method::POST | Method::PUT | Method::DELETE | Method::PATCH)
}
