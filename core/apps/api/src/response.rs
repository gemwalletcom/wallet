use axum::Json;
use axum::body::Body;
use axum::response::{IntoResponse, Response};
use http::StatusCode;
use http::header::CONTENT_TYPE;
use primitives::ResponseResult;
use serde::Serialize;

use crate::error::ApiError;

pub struct ApiResponse<T>(pub ResponseResult<T>);

impl<T> From<T> for ApiResponse<T> {
    fn from(data: T) -> Self {
        ApiResponse(ResponseResult::new(data))
    }
}

impl<T: Serialize> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

pub struct JsonProxyResponse(gem_client::Response);

impl From<gem_client::Response> for JsonProxyResponse {
    fn from(response: gem_client::Response) -> Self {
        Self(response)
    }
}

impl IntoResponse for JsonProxyResponse {
    fn into_response(self) -> Response {
        let gem_client::Response { status, data } = self.0;
        let Some(status) = status.and_then(|status| StatusCode::from_u16(status).ok()) else {
            return ApiError::Internal("Proxy response has no status".to_string()).into_response();
        };
        (status, [(CONTENT_TYPE, "application/json")], Body::from(data)).into_response()
    }
}
