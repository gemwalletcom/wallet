mod client;
mod image_upload;

pub use client::SupportApiClient;
pub use image_upload::SupportImageUploadConfig;
use image_upload::{MAX_SUPPORT_IMAGE_BYTES, validate_support_image_upload};
use primitives::{SupportAction, SupportMessage, SupportMessageInput};
use rocket::{State, get, http::ContentType, post, tokio::sync::Mutex};

use crate::{
    devices::{
        body::{DeviceBody, DeviceJson},
        guard::AuthenticatedDevice,
    },
    responders::{ApiError, ApiResponse},
};

#[get("/devices/support/messages?<from_timestamp>")]
pub async fn get_support_messages(
    device: AuthenticatedDevice,
    from_timestamp: Option<u64>,
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<Vec<SupportMessage>>, ApiError> {
    Ok(client.lock().await.messages(&device.device_row, from_timestamp).await?.into())
}

#[post("/devices/support/messages", format = "json", data = "<input>")]
pub async fn post_support_message(
    device: AuthenticatedDevice,
    input: DeviceJson<SupportMessageInput>,
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<SupportMessage>, ApiError> {
    Ok(client.lock().await.send_message(&device.device_row, input.into_inner()).await?.into())
}

#[post("/devices/support/messages/images?<file_name>", data = "<data>")]
pub async fn post_support_image(
    device: AuthenticatedDevice,
    file_name: Option<String>,
    content_type: &ContentType,
    data: DeviceBody<{ MAX_SUPPORT_IMAGE_BYTES }>,
    upload_config: &State<SupportImageUploadConfig>,
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<SupportMessage>, ApiError> {
    let image = validate_support_image_upload(upload_config, file_name, content_type, data.into_inner())?;
    Ok(client
        .lock()
        .await
        .send_image(&device.device_row, image.data, image.file_name, image.content_type)
        .await?
        .into())
}

#[post("/devices/support/action", format = "json", data = "<action>")]
pub async fn post_support_action(device: AuthenticatedDevice, action: DeviceJson<SupportAction>, client: &State<Mutex<SupportApiClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.run_action(&device.device_row, action.into_inner()).await?.into())
}
