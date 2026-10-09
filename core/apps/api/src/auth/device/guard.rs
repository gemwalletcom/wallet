use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{FromRef, FromRequestParts, State};
use http::header::CONTENT_TYPE;
use http::request::Parts;
use primitives::{Version, WalletId, WalletType};
use serde::de::DeserializeOwned;
use services::devices::{DeviceRecord, DeviceWalletLookup, DevicesClient};

use super::{DeviceError, VerifiedRequest};
use crate::error::ApiError;

pub fn verified(parts: &Parts) -> Result<VerifiedRequest, ApiError> {
    parts.extensions.get::<VerifiedRequest>().cloned().ok_or_else(|| ApiError::internal("Device authentication is not configured for this route"))
}

fn has_json_content_type(parts: &Parts) -> bool {
    parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.split(';').next().unwrap_or_default().trim().to_ascii_lowercase())
        .is_some_and(|media_type| media_type == "application/json" || (media_type.starts_with("application/") && media_type.ends_with("+json")))
}

pub struct VerifiedDeviceId(pub String);

impl<S: Send + Sync> FromRequestParts<S> for VerifiedDeviceId {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(verified(parts)?.device_id))
    }
}

#[derive(Clone)]
pub struct AuthenticatedDevice {
    pub record: DeviceRecord,
}

impl AuthenticatedDevice {
    pub fn version(&self) -> Result<Version, ApiError> {
        self.record.device.version.parse::<Version>().map_err(|error| ApiError::bad_request(error.to_string()))
    }
}

impl<S> FromRequestParts<S> for AuthenticatedDevice
where
    S: Send + Sync,
    Arc<DevicesClient>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request = verified(parts)?;
        let State(devices): State<Arc<DevicesClient>> = State::from_request_parts(parts, state).await.map_err(|_| ApiError::internal("Devices client is not available"))?;
        match devices.find_device_record(&request.device_id).await {
            Ok(Some(record)) => Ok(Self { record }),
            Ok(None) => Err(ApiError::not_found(DeviceError::DeviceNotFound.to_string())),
            Err(_) => Err(ApiError::internal(DeviceError::DatabaseError.to_string())),
        }
    }
}

pub struct AuthenticatedDeviceWallet {
    pub record: DeviceRecord,
    pub wallet_id: i32,
    pub wallet_identifier: WalletId,
    pub wallet_type: WalletType,
}

impl<S> FromRequestParts<S> for AuthenticatedDeviceWallet
where
    S: Send + Sync,
    Arc<DevicesClient>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let request = verified(parts)?;
        let device_id = request.device_id.as_str();
        let wallet_id = request.wallet_id.as_deref().ok_or_else(|| ApiError::unauthorized(DeviceError::MissingWalletId.to_string()))?;
        let State(devices): State<Arc<DevicesClient>> = State::from_request_parts(parts, state).await.map_err(|_| ApiError::internal("Devices client is not available"))?;
        match devices.find_device_wallet(device_id, wallet_id).await {
            Ok(DeviceWalletLookup::Found(record, wallet)) => Ok(Self {
                record,
                wallet_id: wallet.id,
                wallet_identifier: wallet.wallet_id,
                wallet_type: wallet.wallet_type,
            }),
            Ok(DeviceWalletLookup::DeviceNotFound) => Err(ApiError::not_found(DeviceError::DeviceNotFound.to_string())),
            Ok(DeviceWalletLookup::WalletNotFound) => Err(ApiError::not_found(DeviceError::WalletNotFound.to_string())),
            Ok(DeviceWalletLookup::WalletUnavailable) | Err(_) => Err(ApiError::internal(DeviceError::DatabaseError.to_string())),
        }
    }
}

pub struct DeviceJson<T>(T);

impl<T> DeviceJson<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T: DeserializeOwned, S: Send + Sync> FromRequestParts<S> for DeviceJson<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        json_body(parts).map(DeviceJson)
    }
}

pub struct DeviceBody(pub Bytes);

impl<S: Send + Sync> FromRequestParts<S> for DeviceBody {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(verified(parts)?.body))
    }
}

pub fn json_body<T: DeserializeOwned>(parts: &Parts) -> Result<T, ApiError> {
    if !has_json_content_type(parts) {
        return Err(ApiError::unsupported_media_type("Expected request with `Content-Type: application/json`"));
    }
    let request = verified(parts)?;
    serde_json::from_slice::<T>(&request.body).map_err(|_| ApiError::bad_request("Invalid JSON"))
}
