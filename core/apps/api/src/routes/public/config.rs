use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Response};
use http::header::CACHE_CONTROL;
use primitives::config::ConfigResponse;
use services::app::ConfigClient;

use crate::error::ApiError;
use crate::request::ClientIp;
use crate::response::ApiResponse;

pub async fn get_config(ClientIp(ip): ClientIp, State(client): State<Arc<ConfigClient>>) -> Result<Response, ApiError> {
    let config: ApiResponse<ConfigResponse> = client.get_config(&ip.to_string()).await?.into();
    Ok(([(CACHE_CONTROL, "private, no-store")], config).into_response())
}
