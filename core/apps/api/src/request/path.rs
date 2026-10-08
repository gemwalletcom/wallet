use axum::extract::FromRequestParts;
use axum::extract::rejection::PathRejection;
use http::request::Parts;
use serde::de::DeserializeOwned;

use crate::error::ApiError;

pub struct Path<T>(pub T);

impl<T: DeserializeOwned + Send, S: Send + Sync> FromRequestParts<S> for Path<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        axum::extract::Path::<T>::from_request_parts(parts, state).await.map(|axum::extract::Path(value)| Path(value)).map_err(|error| match error {
            PathRejection::FailedToDeserializePathParams(error) => ApiError::UnprocessableEntity(error.kind().to_string()),
            other => ApiError::UnprocessableEntity(other.body_text()),
        })
    }
}
