use rocket::serde::json::Json;
use rocket::{State, post};
use streamer::{FetchListPayload, StreamProducer, StreamProducerQueue};

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/lists/add", format = "json", data = "<request>")]
pub async fn add_list(
    _permission: PermissionAdminWrite,
    request: Json<FetchListPayload>,
    stream_producer: &State<StreamProducer>,
) -> Result<ApiResponse<FetchListPayload>, ApiError> {
    let payload = request.into_inner();
    stream_producer.publish_fetch_list(payload.clone()).await?;
    Ok(payload.into())
}
