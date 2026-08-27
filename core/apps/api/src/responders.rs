use cacher::CacheError;
use fiat::error::FiatQuoteError;
use gem_rewards::{RewardsError, RewardsRedemptionError, UsernameError};
use primitives::{RequestError, ResponseResult};
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{Request, http::Status};
use serde::Serialize;
use storage::DatabaseError;
use strum::ParseError;

pub struct ErrorContext(pub String);

pub fn cache_error(req: &Request<'_>, message: &str) {
    req.local_cache(|| ErrorContext(message.to_string()));
}

fn ok_error_message(error: &(dyn std::error::Error + 'static)) -> Option<String> {
    downcast_error_message::<RewardsError>(error)
        .or_else(|| downcast_error_message::<RewardsRedemptionError>(error))
        .or_else(|| downcast_error_message::<UsernameError>(error))
}

fn downcast_error_message<T>(error: &(dyn std::error::Error + 'static)) -> Option<String>
where
    T: std::error::Error + 'static,
{
    error.downcast_ref::<T>().map(ToString::to_string)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    OkError(String),
    BadRequest(String),
    Forbidden,
    NotFound(String),
    InternalServerError(String),
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message) = match self {
            ApiError::OkError(msg) => (Status::Ok, msg),
            ApiError::BadRequest(msg) => (Status::BadRequest, msg),
            ApiError::Forbidden => (Status::Forbidden, "Forbidden".to_string()),
            ApiError::NotFound(msg) => (Status::NotFound, msg),
            ApiError::InternalServerError(msg) => (Status::InternalServerError, msg),
        };

        let error_response = ResponseResult::<()>::error(message);
        let json_response = Json(error_response);

        Response::build_from(json_response.respond_to(request)?).status(status).ok()
    }
}

impl From<CacheError> for ApiError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::NotFound { .. } | CacheError::ResourceNotFound(_) => ApiError::NotFound(error.to_string()),
            CacheError::KeyNotFound(_) => ApiError::InternalServerError("Unexpected cache miss".to_string()),
        }
    }
}

impl From<ParseError> for ApiError {
    fn from(error: ParseError) -> Self {
        ApiError::NotFound(format!("Invalid parameter: {}", error))
    }
}

impl From<DatabaseError> for ApiError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound { .. } => ApiError::NotFound(error.to_string()),
            DatabaseError::Error(msg) => ApiError::InternalServerError(msg),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::InternalServerError(error.to_string())
    }
}

impl From<swapper::SwapperError> for ApiError {
    fn from(error: swapper::SwapperError) -> Self {
        ApiError::InternalServerError(error.to_string())
    }
}

impl From<FiatQuoteError> for ApiError {
    fn from(error: FiatQuoteError) -> Self {
        ApiError::BadRequest(error.to_string())
    }
}

impl From<RequestError> for ApiError {
    fn from(_: RequestError) -> Self {
        ApiError::Forbidden
    }
}

impl From<RewardsError> for ApiError {
    fn from(error: RewardsError) -> Self {
        ApiError::OkError(error.to_string())
    }
}

impl From<RewardsRedemptionError> for ApiError {
    fn from(error: RewardsRedemptionError) -> Self {
        ApiError::OkError(error.to_string())
    }
}

impl From<UsernameError> for ApiError {
    fn from(error: UsernameError) -> Self {
        ApiError::OkError(error.to_string())
    }
}

impl From<Box<dyn std::error::Error + Send + Sync>> for ApiError {
    fn from(error: Box<dyn std::error::Error + Send + Sync>) -> Self {
        let mut current_error: &(dyn std::error::Error + 'static) = error.as_ref();
        loop {
            if let Some(cache_error) = current_error.downcast_ref::<CacheError>() {
                return cache_error.clone().into();
            }
            if let Some(db_error) = current_error.downcast_ref::<DatabaseError>() {
                return db_error.clone().into();
            }
            if let Some(request_error) = current_error.downcast_ref::<RequestError>() {
                return (*request_error).into();
            }
            if let Some(fiat_error) = current_error.downcast_ref::<FiatQuoteError>() {
                return fiat_error.clone().into();
            }
            if let Some(message) = ok_error_message(current_error) {
                return ApiError::OkError(message);
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::InternalServerError(format!("{}", error))
    }
}

pub struct ApiResponse<T>(pub ResponseResult<T>);

impl<T> From<T> for ApiResponse<T> {
    fn from(data: T) -> Self {
        ApiResponse(ResponseResult::new(data))
    }
}

impl<'r, T: Serialize> Responder<'r, 'static> for ApiResponse<T> {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        Json(self.0).respond_to(request)
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;
    use cacher::CacheError;
    use gem_rewards::{RewardsError, RewardsRedemptionError};
    use primitives::RequestError;
    use storage::DatabaseError;

    #[test]
    fn test_cache_not_found_maps_to_public_not_found() {
        let error = ApiError::from(CacheError::not_found("FiatQuote", "abc"));
        assert_eq!(error, ApiError::NotFound("FiatQuote abc not found".to_string()));
    }

    #[test]
    fn test_cache_key_not_found_maps_to_internal_server_error() {
        let error = ApiError::from(CacheError::KeyNotFound("fiat:quote:abc".to_string()));
        assert_eq!(error, ApiError::InternalServerError("Unexpected cache miss".to_string()));
    }

    #[test]
    fn test_boxed_database_not_found_hides_internal_lookup() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(DatabaseError::not_found_internal("Device", "1"));
        assert_eq!(ApiError::from(error), ApiError::NotFound("Device not found".to_string()));
    }

    #[test]
    fn test_boxed_rewards_error_maps_to_ok_error() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(RewardsError::Username("Daily username creation limit has been reached".to_string()));
        assert_eq!(ApiError::from(error), ApiError::OkError("Daily username creation limit has been reached".to_string()));
    }

    #[test]
    fn test_boxed_rewards_redemption_error_maps_to_ok_error() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(RewardsRedemptionError::LimitReached);
        assert_eq!(ApiError::from(error), ApiError::OkError("Redemption limit reached".to_string()));
    }

    #[test]
    fn test_request_error_maps_to_forbidden() {
        assert_eq!(ApiError::from(RequestError::Forbidden), ApiError::Forbidden);
    }
}
