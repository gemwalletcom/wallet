use rocket::serde::json::Json;
use rocket::{State, post};
use services::indexer::{FetchListPayload, IndexerClient};

use crate::api_clients::PermissionAdminWrite;
use crate::responders::{ApiError, ApiResponse};

#[post("/lists/add", format = "json", data = "<request>")]
pub async fn add_list(_permission: PermissionAdminWrite, request: Json<FetchListPayload>, client: &State<IndexerClient>) -> Result<ApiResponse<FetchListPayload>, ApiError> {
    let payload = request.into_inner();
    client.fetch_list(payload.clone()).await?;
    Ok(payload.into())
}
