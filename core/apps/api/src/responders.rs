use fiat::error::FiatQuoteError;
use gem_auth::JwtError;
use gem_client::ClientError;
use gem_tracing::error_fields;
use localizer::LanguageLocalizer;
use primitives::{RequestError, ResponseResult};
use rewards::{RewardsError, RewardsRedemptionError, UsernameError};
use rocket::response::{Responder, Response};
use rocket::serde::json::Json;
use rocket::{Request, http::Status};
use serde::Serialize;
use services::{CacheError, DatabaseError};
use strum::ParseError;

pub struct ErrorContext(pub String);

pub fn cache_error(req: &Request<'_>, message: &str) {
    req.local_cache(|| ErrorContext(message.to_string()));
}

pub fn localized_fiat_error(error: Box<dyn std::error::Error + Send + Sync>, locale: &str) -> ApiError {
    let localizer = LanguageLocalizer::new_with_language(locale);
    if let Some(error) = error.downcast_ref::<RequestError>() {
        return match error {
            RequestError::LimitReached => ApiError::OkError(localizer.fiat_error_limit_reached()),
            RequestError::Forbidden => ApiError::BadRequest(localizer.fiat_error_quote_unavailable()),
        };
    }
    if error.downcast_ref::<FiatQuoteError>().is_some() {
        return ApiError::BadRequest(localizer.errors_generic());
    }
    ApiError::from(error)
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
    Internal(String),
}

pub const INTERNAL_ERROR_MESSAGE: &str = "Internal server error";

impl ApiError {
    fn public(self) -> (Status, String, Option<String>) {
        match self {
            ApiError::OkError(msg) => (Status::Ok, msg, None),
            ApiError::BadRequest(msg) => (Status::BadRequest, msg, None),
            ApiError::Forbidden => (Status::Forbidden, "Forbidden".to_string(), None),
            ApiError::NotFound(msg) => (Status::NotFound, msg, None),
            ApiError::Internal(detail) => (Status::InternalServerError, INTERNAL_ERROR_MESSAGE.to_string(), Some(detail)),
        }
    }
}

impl<'r> Responder<'r, 'static> for ApiError {
    fn respond_to(self, request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let (status, message, detail) = self.public();
        if let Some(detail) = detail {
            let uri = request.uri().to_string();
            let user_agent = request.headers().get_one("User-Agent").unwrap_or("unknown");
            error_fields!("Request failed", uri = uri, status = status.code, error = detail, user_agent = user_agent);
        }

        let error_response = ResponseResult::<()>::error(message);
        let json_response = Json(error_response);

        Response::build_from(json_response.respond_to(request)?).status(status).ok()
    }
}

impl From<CacheError> for ApiError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::NotFound { .. } | CacheError::ResourceNotFound(_) => ApiError::NotFound(error.to_string()),
            CacheError::KeyNotFound(_) => ApiError::Internal("Unexpected cache miss".to_string()),
        }
    }
}

impl From<JwtError> for ApiError {
    fn from(error: JwtError) -> Self {
        ApiError::Internal(error.to_string())
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
            DatabaseError::ConnectionPool => ApiError::Internal(error.to_string()),
            DatabaseError::Error(msg) => ApiError::Internal(msg),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(error: serde_json::Error) -> Self {
        ApiError::Internal(error.to_string())
    }
}

impl From<swapper::SwapperError> for ApiError {
    fn from(error: swapper::SwapperError) -> Self {
        ApiError::Internal(error.to_string())
    }
}

impl From<FiatQuoteError> for ApiError {
    fn from(error: FiatQuoteError) -> Self {
        ApiError::BadRequest(error.to_string())
    }
}

impl From<RequestError> for ApiError {
    fn from(error: RequestError) -> Self {
        match error {
            RequestError::Forbidden => ApiError::Forbidden,
            RequestError::LimitReached => ApiError::OkError(error.to_string()),
        }
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
            if let Some(ClientError::Http { status, body }) = current_error.downcast_ref::<ClientError>() {
                return ApiError::Internal(format!("upstream status {status}: {}", String::from_utf8_lossy(body)));
            }
            if let Some(message) = ok_error_message(current_error) {
                return ApiError::OkError(message);
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::Internal(error.to_string())
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
    use super::{ApiError, INTERNAL_ERROR_MESSAGE};
    use gem_client::ClientError;
    use primitives::RequestError;
    use rewards::{RewardsError, RewardsRedemptionError};
    use rocket::http::Status;
    use services::{CacheError, DatabaseError};

    #[test]
    fn test_a_fiat_error_reads_in_the_device_language() {
        let limited = super::localized_fiat_error(Box::new(RequestError::LimitReached), "de");
        assert_eq!(limited, ApiError::OkError("Zu viele Angebotsanfragen. Bitte versuchen Sie es in ein paar Minuten erneut.".to_string()));
        assert_eq!(
            super::localized_fiat_error(Box::new(RequestError::Forbidden), "en"),
            ApiError::BadRequest("This quote is no longer available. Please try again.".to_string()),
            "a quote that expired says so instead of a bare Forbidden"
        );
        assert_eq!(
            super::localized_fiat_error(Box::new(fiat::error::FiatQuoteError::InvalidRequest("Missing network".to_string())), "en"),
            ApiError::BadRequest("An unexpected error occurred. Please try again later.".to_string())
        );
        assert_eq!(super::localized_fiat_error("connection refused".into(), "en"), ApiError::Internal("connection refused".to_string()));
    }

    #[test]
    fn test_cache_not_found_maps_to_public_not_found() {
        let error = ApiError::from(CacheError::not_found("FiatQuote", "abc"));
        assert_eq!(error, ApiError::NotFound("FiatQuote abc not found".to_string()));
    }

    #[test]
    fn test_cache_key_not_found_maps_to_internal_server_error() {
        let error = ApiError::from(CacheError::KeyNotFound("fiat:quote:abc".to_string()));
        assert_eq!(error, ApiError::Internal("Unexpected cache miss".to_string()));
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
    fn test_boxed_upstream_failure_is_logged_not_returned() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(ClientError::Http {
            status: 500,
            body: b"NoMethodError (undefined method '[]' for nil)".to_vec(),
        });

        let (status, message, detail) = ApiError::from(error).public();

        assert_eq!(status, Status::InternalServerError);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some("upstream status 500: NoMethodError (undefined method '[]' for nil)"));
    }

    #[test]
    fn test_database_error_is_logged_not_returned() {
        let raw = "duplicate key value violates unique constraint \"devices_device_id_key\"";
        let (status, message, detail) = ApiError::from(DatabaseError::Error(raw.to_string())).public();

        assert_eq!(status, Status::InternalServerError);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some(raw));
    }

    #[test]
    fn test_unknown_boxed_error_is_logged_not_returned() {
        let error: Box<dyn std::error::Error + Send + Sync> = "connection refused".into();
        let (status, message, detail) = ApiError::from(error).public();

        assert_eq!(status, Status::InternalServerError);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some("connection refused"));
    }

    #[test]
    fn test_boxed_database_error_is_logged_not_returned() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(DatabaseError::Error("relation \"devices\" does not exist".to_string()));
        let (_, message, _) = ApiError::from(error).public();

        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
    }

    #[test]
    fn test_user_facing_errors_keep_their_message() {
        assert_eq!(ApiError::OkError("Rate limit reached".to_string()).public(), (Status::Ok, "Rate limit reached".to_string(), None));
        assert_eq!(ApiError::NotFound("Device not found".to_string()).public(), (Status::NotFound, "Device not found".to_string(), None));
    }

    #[test]
    fn test_request_error_maps_to_forbidden() {
        assert_eq!(ApiError::from(RequestError::Forbidden), ApiError::Forbidden);
    }

    #[test]
    fn test_boxed_limit_reached_maps_to_ok_error() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(RequestError::LimitReached);
        assert_eq!(ApiError::from(error), ApiError::OkError("Rate limit reached".to_string()));
    }
}
