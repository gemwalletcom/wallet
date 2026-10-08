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
            Err(JsonRejection::JsonDataError(error)) => Err(ApiError::UnprocessableEntity(error.body_text())),
            Err(JsonRejection::JsonSyntaxError(_)) => Err(ApiError::BadRequest("Invalid JSON".to_string())),
            Err(JsonRejection::MissingJsonContentType(error)) => Err(ApiError::UnsupportedMediaType(error.body_text())),
            Err(error) => Err(ApiError::status(error.status(), error.body_text())),
        }
    }
}
