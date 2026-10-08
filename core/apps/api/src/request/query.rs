use axum::extract::FromRequestParts;
use http::request::Parts;
use serde::de::DeserializeOwned;

use crate::error::ApiError;

const QUERY_ERROR_PREFIX: &str = "Failed to deserialize query string: ";

pub struct Query<T>(pub T);

impl<T: DeserializeOwned, S: Send + Sync> FromRequestParts<S> for Query<T> {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        axum::extract::Query::<T>::try_from_uri(&parts.uri)
            .map(|axum::extract::Query(value)| Query(value))
            .map_err(|error| ApiError::UnprocessableEntity(error.body_text().trim_start_matches(QUERY_ERROR_PREFIX).to_string()))
    }
}
