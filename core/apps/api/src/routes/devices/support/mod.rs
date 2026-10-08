mod image_upload;

use std::sync::Arc;

use axum::extract::State;
use http::HeaderMap;
use http::header::CONTENT_TYPE;
use image_upload::validate_support_image_upload;
pub use image_upload::{MAX_SUPPORT_IMAGE_BYTES, SupportImageUploadConfig};
use primitives::{SupportAction, SupportMessage, SupportMessageInput};
use serde::Deserialize;
use services::support::SupportApiClient;

use crate::auth::device::{AuthenticatedDevice, DeviceBody, DeviceJson};
use crate::error::ApiError;
use crate::request::{FromTimestampQuery, Query};
use crate::response::ApiResponse;

#[derive(Deserialize)]
pub struct ImageQuery {
    file_name: Option<String>,
}

pub async fn get_messages(device: AuthenticatedDevice, Query(query): Query<FromTimestampQuery>, State(client): State<Arc<SupportApiClient>>) -> Result<ApiResponse<Vec<SupportMessage>>, ApiError> {
    Ok(client.messages(&device.record, query.from_timestamp).await?.into())
}

pub async fn post_message(device: AuthenticatedDevice, State(client): State<Arc<SupportApiClient>>, input: DeviceJson<SupportMessageInput>) -> Result<ApiResponse<SupportMessage>, ApiError> {
    Ok(client.send_message(&device.record, input.into_inner()).await?.into())
}

pub async fn post_image(
    device: AuthenticatedDevice,
    Query(query): Query<ImageQuery>,
    headers: HeaderMap,
    State(config): State<Arc<SupportImageUploadConfig>>,
    State(client): State<Arc<SupportApiClient>>,
    body: DeviceBody,
) -> Result<ApiResponse<SupportMessage>, ApiError> {
    let content_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::BadRequest("Missing header: Content-Type".to_string()))?;
    let image = validate_support_image_upload(&config, query.file_name, content_type, body.0.to_vec())?;
    Ok(client.send_image(&device.record, image.data, image.file_name, image.content_type).await?.into())
}

pub async fn post_action(device: AuthenticatedDevice, State(client): State<Arc<SupportApiClient>>, action: DeviceJson<SupportAction>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.run_action(&device.record, action.into_inner()).await?.into())
}
