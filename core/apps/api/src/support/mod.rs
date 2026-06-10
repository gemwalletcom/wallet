mod client;

pub use client::SupportApiClient;
use primitives::{SupportAction, SupportMessage, SupportMessageInput};
use rocket::{State, get, http::ContentType, post, tokio::sync::Mutex};

use crate::{
    devices::{
        body::{DeviceBody, DeviceJson},
        guard::AuthenticatedDevice,
    },
    responders::{ApiError, ApiResponse},
};

const MAX_SUPPORT_IMAGE_BYTES: u64 = 10 * 1024 * 1024;

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
    client: &State<Mutex<SupportApiClient>>,
) -> Result<ApiResponse<SupportMessage>, ApiError> {
    if content_type.top() != "image" {
        return Err(ApiError::BadRequest("Only image uploads are supported".to_string()));
    }

    let file_name = file_name.unwrap_or_else(|| format!("support.{}", content_type.sub()));
    Ok(client
        .lock()
        .await
        .send_image(&device.device_row, data.into_inner(), file_name, content_type.to_string())
        .await?
        .into())
}

#[post("/devices/support/action", format = "json", data = "<action>")]
pub async fn post_support_action(device: AuthenticatedDevice, action: DeviceJson<SupportAction>, client: &State<Mutex<SupportApiClient>>) -> Result<ApiResponse<bool>, ApiError> {
    Ok(client.lock().await.run_action(&device.device_row, action.into_inner()).await?.into())
}
