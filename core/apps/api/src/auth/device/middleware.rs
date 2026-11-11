use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::{Body, Bytes};
use axum::extract::{OriginalUri, Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use gem_auth::{AUTHORIZATION_HEADER, DeviceSignature, DeviceSignatureHeaders, device_auth_message, device_body_hash, parse_device_auth, verify_device_signature};
use http::Uri;
use http::request::Parts;
use http_body_util::LengthLimitError;
use primitives::hex;

use super::{DEVICE_ID_LENGTH, DeviceAuthConfig, DeviceError};
use crate::error::ApiError;

#[derive(Clone, Copy)]
pub enum SignedPath {
    PathOnly,
    PathAndQuery,
}

#[derive(Clone, Copy)]
pub enum DeviceAuthScheme {
    Gem(SignedPath),
    HttpSignature,
}

#[derive(Clone)]
pub struct DeviceAuth {
    pub config: Arc<DeviceAuthConfig>,
    pub limit: usize,
    pub scheme: DeviceAuthScheme,
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

struct SignedRequest {
    device_id: String,
    wallet_id: Option<String>,
    body_hash: String,
}

async fn verify(auth: &DeviceAuth, uri: &Uri, request: Request) -> Result<Request, ApiError> {
    let (mut parts, body) = request.into_parts();
    let signed = match auth.scheme {
        DeviceAuthScheme::Gem(signed_path) => verify_authorization(auth, uri, &parts, signed_path)?,
        DeviceAuthScheme::HttpSignature => verify_http_signature(auth, uri, &parts)?,
    };
    if signed.device_id.len() != DEVICE_ID_LENGTH {
        return Err(ApiError::unauthorized(DeviceError::InvalidDeviceId.to_string()));
    }
    let body = match axum::body::to_bytes(body, auth.limit).await {
        Ok(body) => body,
        Err(error) => {
            if error.into_inner().is::<LengthLimitError>() {
                return Err(ApiError::payload_too_large("Request body too large"));
            }
            return Err(ApiError::bad_request("Failed to read body"));
        }
    };
    if device_body_hash(&body) != signed.body_hash {
        return Err(ApiError::bad_request("Body hash mismatch"));
    }
    parts.extensions.insert(VerifiedRequest {
        device_id: signed.device_id,
        wallet_id: signed.wallet_id,
        body: body.clone(),
    });
    Ok(Request::from_parts(parts, Body::from(body)))
}

fn verify_authorization(auth: &DeviceAuth, uri: &Uri, parts: &Parts, signed_path: SignedPath) -> Result<SignedRequest, ApiError> {
    let header = parts
        .headers
        .get(AUTHORIZATION_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::unauthorized(DeviceError::MissingHeader(AUTHORIZATION_HEADER).to_string()))?;
    let payload = parse_device_auth(header).ok_or_else(|| ApiError::unauthorized(DeviceError::InvalidAuthorizationFormat.to_string()))?;
    let timestamp_ms: u64 = payload.timestamp.parse().map_err(|_| ApiError::unauthorized(DeviceError::InvalidTimestamp.to_string()))?;
    check_timestamp(auth, timestamp_ms)?;
    let signed_path = match signed_path {
        SignedPath::PathOnly => uri.path(),
        SignedPath::PathAndQuery => uri.path_and_query().map(http::uri::PathAndQuery::as_str).unwrap_or("/"),
    };
    let wallet_id = payload.wallet_id.as_deref().unwrap_or("");
    let message = device_auth_message(&payload.timestamp, parts.method.as_str(), signed_path, wallet_id, &payload.body_hash);
    if !verify_device_signature(&payload.device_id, &message, &payload.signature) {
        return Err(ApiError::unauthorized(DeviceError::InvalidSignature.to_string()));
    }
    Ok(SignedRequest {
        device_id: payload.device_id,
        wallet_id: payload.wallet_id,
        body_hash: payload.body_hash,
    })
}

fn verify_http_signature(auth: &DeviceAuth, uri: &Uri, parts: &Parts) -> Result<SignedRequest, ApiError> {
    let headers = DeviceSignatureHeaders::from_fields(|name| parts.headers.get(name).and_then(|value| value.to_str().ok())).map_err(|name| ApiError::unauthorized(DeviceError::MissingHeader(name).to_string()))?;
    let signature = DeviceSignature::parse(&headers).ok_or_else(|| ApiError::unauthorized(DeviceError::InvalidSignatureFormat.to_string()))?;
    check_timestamp(auth, signature.created.saturating_mul(1000))?;
    if !signature.verify(parts.method.as_str(), uri.path(), uri.query()) {
        return Err(ApiError::unauthorized(DeviceError::InvalidSignature.to_string()));
    }
    Ok(SignedRequest {
        device_id: signature.device_id,
        wallet_id: signature.wallet_id,
        body_hash: hex::encode(signature.body_sha256),
    })
}

fn check_timestamp(auth: &DeviceAuth, timestamp_ms: u64) -> Result<(), ApiError> {
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH).map_err(|_| ApiError::unauthorized(DeviceError::InvalidTimestamp.to_string()))?.as_millis() as u64;
    if now_ms.abs_diff(timestamp_ms) > auth.config.tolerance.as_millis() as u64 {
        return Err(ApiError::unauthorized(DeviceError::TimestampExpired.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    use axum::Router;
    use axum::extract::Extension;
    use axum::middleware::from_fn_with_state;
    use axum::routing::post;
    use gem_auth::sign_device_request;
    use http::StatusCode;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use super::*;

    const PRIVATE_KEY: [u8; 32] = [1u8; 32];
    const TARGET: &str = "/v4/devices/transactions?asset_id=near&limit=5";
    const WALLET_ID: &str = "view_near_alice.near";
    const BODY: &[u8] = br#"{"limit":5}"#;

    fn router() -> Router {
        let auth = DeviceAuth {
            config: Arc::new(DeviceAuthConfig::new(Duration::from_secs(60))),
            limit: 1024,
            scheme: DeviceAuthScheme::HttpSignature,
        };
        let routes = Router::new().route("/transactions", post(|Extension(request): Extension<VerifiedRequest>| async move { request.wallet_id.unwrap_or_default() }));
        Router::new().nest("/v4/devices", routes.layer(from_fn_with_state(auth, device_auth)))
    }

    fn now() -> u64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
    }

    fn signed(created: u64) -> HashMap<String, String> {
        sign_device_request(&PRIVATE_KEY, "POST", TARGET, Some(WALLET_ID), BODY, created).unwrap()
    }

    fn http_request(uri: &str, headers: HashMap<String, String>, body: &'static [u8]) -> Request {
        headers.into_iter().fold(Request::post(uri), |request, (name, value)| request.header(name, value)).body(Body::from(body)).unwrap()
    }

    #[tokio::test]
    async fn test_http_signature_verified_request() {
        let response = router().oneshot(http_request(TARGET, signed(now()), BODY)).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.into_body().collect().await.unwrap().to_bytes(), WALLET_ID);
    }

    #[tokio::test]
    async fn test_http_signature_rejections() {
        for (request, status) in [
            (http_request("/v4/devices/transactions?asset_id=bitcoin&limit=5", signed(now()), BODY), StatusCode::UNAUTHORIZED),
            (http_request(TARGET, signed(now() - 600), BODY), StatusCode::UNAUTHORIZED),
            (http_request(TARGET, HashMap::new(), BODY), StatusCode::UNAUTHORIZED),
            (http_request(TARGET, signed(now()), br#"{"limit":500}"#), StatusCode::BAD_REQUEST),
        ] {
            assert_eq!(router().oneshot(request).await.unwrap().status(), status);
        }
    }
}
