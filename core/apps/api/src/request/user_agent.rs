use axum::extract::FromRequestParts;
use http::header::USER_AGENT;
use http::request::Parts;

use crate::error::ApiError;

pub struct UserAgent(pub String);

impl<S: Send + Sync> FromRequestParts<S> for UserAgent {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .headers
            .get(USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| Self(value.to_string()))
            .ok_or_else(|| ApiError::BadRequest("Missing header: User-Agent".to_string()))
    }
}
