use std::time::{SystemTime, UNIX_EPOCH};

use gem_auth::{AUTHORIZATION_HEADER, DeviceAuthPayload, device_auth_message, device_body_hash, parse_device_auth, verify_device_signature};
use rocket::http::Status;
use rocket::outcome::Outcome::Success;
use rocket::{Request, State};

use crate::devices::auth_config::AuthConfig;
use crate::devices::error::DeviceError;

fn verify_request_signature(req: &Request<'_>, components: &DeviceAuthPayload, tolerance_ms: u64) -> Result<(), (Status, String)> {
    let timestamp_ms: u64 = components
        .timestamp
        .parse()
        .map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| (Status::Unauthorized, DeviceError::InvalidTimestamp.to_string()))?
        .as_millis() as u64;

    if now_ms.abs_diff(timestamp_ms) > tolerance_ms {
        return Err((Status::Unauthorized, DeviceError::TimestampExpired.to_string()));
    }

    let method = req.method().as_str();
    let path = req.uri().path().as_str();
    let wallet_id = components.wallet_id.as_deref().unwrap_or("");
    let message = device_auth_message(&components.timestamp, method, path, wallet_id, &components.body_hash);

    if !verify_device_signature(&components.device_id, &message, &components.signature) {
        return Err((Status::Unauthorized, DeviceError::InvalidSignature.to_string()));
    }

    Ok(())
}

pub(crate) async fn verify_request_auth<'r>(req: &'r Request<'_>) -> Result<&'r DeviceAuthPayload, (Status, String)> {
    let Success(config) = req.guard::<&State<AuthConfig>>().await else {
        return Err((Status::InternalServerError, "AuthConfig not configured".to_string()));
    };

    let tolerance_ms = config.tolerance.as_millis() as u64;
    req.local_cache_async(async {
        let auth_value = req
            .headers()
            .get_one(AUTHORIZATION_HEADER)
            .ok_or_else(|| (Status::Unauthorized, DeviceError::MissingHeader(AUTHORIZATION_HEADER).to_string()))?;
        let components = parse_device_auth(auth_value).ok_or_else(|| (Status::Unauthorized, DeviceError::InvalidAuthorizationFormat.to_string()))?;
        verify_request_signature(req, &components, tolerance_ms)?;
        Ok(components)
    })
    .await
    .as_ref()
    .map_err(|(status, message)| (*status, message.clone()))
}

pub(crate) async fn verify_request_body_hash(req: &Request<'_>, body: &[u8]) -> Result<(), (Status, String)> {
    let auth = verify_request_auth(req).await?;

    if device_body_hash(body) != auth.body_hash {
        return Err((Status::BadRequest, "Body hash mismatch".to_string()));
    }

    Ok(())
}
