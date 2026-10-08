use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequest, Request};
use serde::de::DeserializeOwned;

use crate::error::ApiError;

pub struct Json<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequest<S> for Json<T> {
    type Rejection = ApiError;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(Json(value)),
            Err(JsonRejection::JsonDataError(error)) => Err(ApiError::unprocessable(error.body_text())),
            Err(JsonRejection::JsonSyntaxError(_)) => Err(ApiError::bad_request("Invalid JSON")),
            Err(JsonRejection::MissingJsonContentType(error)) => Err(ApiError::unsupported_media_type(error.body_text())),
            Err(error) => Err(ApiError::new(error.status(), error.body_text())),
        }
    }
}
