use std::sync::Arc;

use axum::extract::State;
use gem_tracing::error_with_fields;
use primitives::device::Device;
use services::devices::DevicesClient;
use services::support::SupportApiClient;

use crate::auth::device::{AuthenticatedDevice, DeviceJson, VerifiedDeviceId};
use crate::error::ApiError;
use crate::response::ApiResponse;

pub async fn register(device_id: VerifiedDeviceId, State(client): State<Arc<DevicesClient>>, device: DeviceJson<Device>) -> Result<ApiResponse<Device>, ApiError> {
    let device = device.into_inner();
    if device.id != device_id.0 {
        return Err(ApiError::BadRequest("Device id mismatch".to_string()));
    }
    Ok(client.add_device(device).await?.into())
}

pub async fn get_device(device: AuthenticatedDevice, State(client): State<Arc<DevicesClient>>) -> Result<ApiResponse<Device>, ApiError> {
    Ok(client.get_device(&device.record.device.id).await?.into())
}

pub async fn is_registered(device_id: VerifiedDeviceId, State(client): State<Arc<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.is_device_registered(&device_id.0).await?.into())
}

pub async fn update(device: AuthenticatedDevice, State(client): State<Arc<DevicesClient>>, State(support): State<Arc<SupportApiClient>>, input: DeviceJson<Device>) -> Result<ApiResponse<Device>, ApiError> {
    let input = input.into_inner();
    if input.id != device.record.device.id {
        return Err(ApiError::BadRequest("Device id mismatch".to_string()));
    }
    if let Err(error) = support.update_contact(device.record.id, &input).await {
        error_with_fields!("support contact update failed", &*error, device_id = device.record.id);
    }
    Ok(client.update_device(input).await?.into())
}

pub async fn send_push_notification(device: AuthenticatedDevice, State(client): State<Arc<DevicesClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.send_push_notification_device(&device.record.device.id).await?.into())
}
