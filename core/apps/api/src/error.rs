use axum::response::{IntoResponse, Response};
use fiat::error::FiatQuoteError;
use gem_auth::JwtError;
use gem_client::ClientError;
use http::StatusCode;
use http_server::ErrorBody;
use localizer::LanguageLocalizer;
use primitives::RequestError;
use services::fiat::FiatServiceError;
use services::rewards::RewardsServiceError;
use services::{CacheError, DatabaseError};
use strum::ParseError;

pub const INTERNAL_ERROR_MESSAGE: &str = "Internal server error";

#[derive(Clone)]
pub struct ErrorContext {
    pub message: String,
    pub detail: Option<String>,
}

pub fn localized_fiat_error(error: FiatServiceError, locale: &str) -> ApiError {
    let localizer = LanguageLocalizer::new_with_language(locale);
    match error {
        FiatServiceError::Request(RequestError::LimitReached) => ApiError::OkError(localizer.fiat_error_limit_reached()),
        FiatServiceError::Request(RequestError::Forbidden) => ApiError::BadRequest(localizer.fiat_error_quote_unavailable()),
        FiatServiceError::Quote(FiatQuoteError::RegionUnavailable) => ApiError::OkError(localizer.fiat_error_region_unavailable()),
        FiatServiceError::Quote(FiatQuoteError::MinimumAmount(_) | FiatQuoteError::UnsupportedState(_) | FiatQuoteError::InvalidRequest(_) | FiatQuoteError::InvalidWebhook) => ApiError::BadRequest(localizer.errors_generic()),
        FiatServiceError::Storage(error) => error.into(),
        FiatServiceError::Internal(error) => ApiError::Internal(error.to_string()),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApiError {
    OkError(String),
    BadRequest(String),
    Unauthorized(String),
    Forbidden,
    NotFound(String),
    PayloadTooLarge(String),
    UnsupportedMediaType(String),
    UnprocessableEntity(String),
    Internal(String),
}

impl ApiError {
    pub fn from_status(status: StatusCode) -> Self {
        Self::status(status, format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown")))
    }

    pub fn status(status: StatusCode, message: impl Into<String>) -> Self {
        let message = message.into();
        match status {
            StatusCode::OK => ApiError::OkError(message),
            StatusCode::BAD_REQUEST => ApiError::BadRequest(message),
            StatusCode::UNAUTHORIZED => ApiError::Unauthorized(message),
            StatusCode::FORBIDDEN => ApiError::Forbidden,
            StatusCode::NOT_FOUND => ApiError::NotFound(message),
            StatusCode::PAYLOAD_TOO_LARGE => ApiError::PayloadTooLarge(message),
            StatusCode::UNSUPPORTED_MEDIA_TYPE => ApiError::UnsupportedMediaType(message),
            StatusCode::UNPROCESSABLE_ENTITY => ApiError::UnprocessableEntity(message),
            _ => ApiError::Internal(message),
        }
    }

    pub fn public(self) -> (StatusCode, String, Option<String>) {
        match self {
            ApiError::OkError(message) => (StatusCode::OK, message, None),
            ApiError::BadRequest(message) => (StatusCode::BAD_REQUEST, message, None),
            ApiError::Unauthorized(message) => (StatusCode::UNAUTHORIZED, message, None),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "Forbidden".to_string(), None),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, message, None),
            ApiError::PayloadTooLarge(message) => (StatusCode::PAYLOAD_TOO_LARGE, message, None),
            ApiError::UnsupportedMediaType(message) => (StatusCode::UNSUPPORTED_MEDIA_TYPE, message, None),
            ApiError::UnprocessableEntity(message) => (StatusCode::UNPROCESSABLE_ENTITY, message, None),
            ApiError::Internal(detail) => (StatusCode::INTERNAL_SERVER_ERROR, INTERNAL_ERROR_MESSAGE.to_string(), Some(detail)),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message, detail) = self.public();
        let mut response = ErrorBody::new(status, message.clone()).into_response();
        response.extensions_mut().insert(ErrorContext { message, detail });
        response
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

impl From<FiatServiceError> for ApiError {
    fn from(error: FiatServiceError) -> Self {
        match error {
            FiatServiceError::Request(error) => error.into(),
            FiatServiceError::Quote(error) => error.into(),
            FiatServiceError::Storage(error) => error.into(),
            FiatServiceError::Internal(error) => ApiError::Internal(error.to_string()),
        }
    }
}

impl From<RewardsServiceError> for ApiError {
    fn from(error: RewardsServiceError) -> Self {
        match error {
            RewardsServiceError::Rejected(error) => ApiError::OkError(error.to_string()),
            RewardsServiceError::Storage(error) => error.into(),
            RewardsServiceError::Internal(error) => ApiError::Internal(error.to_string()),
        }
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
            if let Some(ClientError::Http { status, body }) = current_error.downcast_ref::<ClientError>() {
                return ApiError::Internal(format!("upstream status {status}: {}", String::from_utf8_lossy(body)));
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::Internal(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiError, INTERNAL_ERROR_MESSAGE};
    use fiat::error::FiatQuoteError;
    use gem_client::ClientError;
    use http::StatusCode;
    use primitives::RequestError;
    use rewards::RewardsError;
    use services::fiat::FiatServiceError;
    use services::rewards::RewardsServiceError;
    use services::{CacheError, DatabaseError};

    #[test]
    fn test_a_fiat_error_reads_in_the_device_language() {
        let unavailable = super::localized_fiat_error(FiatServiceError::Quote(FiatQuoteError::RegionUnavailable), "en");
        assert_eq!(unavailable.public(), (StatusCode::OK, "Not available in your region.".to_string(), None));
        let limited = super::localized_fiat_error(FiatServiceError::Request(RequestError::LimitReached), "de");
        assert_eq!(limited, ApiError::OkError("Zu viele Angebotsanfragen. Bitte versuchen Sie es in ein paar Minuten erneut.".to_string()));
        assert_eq!(
            super::localized_fiat_error(FiatServiceError::Request(RequestError::Forbidden), "en"),
            ApiError::BadRequest("This quote is no longer available. Please try again.".to_string()),
            "a quote that expired says so instead of a bare Forbidden"
        );
        assert_eq!(
            super::localized_fiat_error(FiatServiceError::Quote(FiatQuoteError::InvalidRequest("Missing network".to_string())), "en"),
            ApiError::BadRequest("An unexpected error occurred. Please try again later.".to_string())
        );
        assert_eq!(super::localized_fiat_error(FiatServiceError::Internal("connection refused".into()), "en"), ApiError::Internal("connection refused".to_string()));
        assert_eq!(
            super::localized_fiat_error(FiatServiceError::Storage(DatabaseError::not_found("Asset", "btc")), "en"),
            ApiError::NotFound("Asset btc not found".to_string())
        );
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
    fn test_rewards_service_error_mapping() {
        let rejected = RewardsServiceError::Rejected(RewardsError::Username("Daily username creation limit has been reached".to_string()));
        assert_eq!(ApiError::from(rejected), ApiError::OkError("Daily username creation limit has been reached".to_string()));
        assert_eq!(
            ApiError::from(RewardsServiceError::Storage(DatabaseError::not_found_internal("Rewards", "1"))),
            ApiError::NotFound("Rewards not found".to_string())
        );
        assert_eq!(ApiError::from(RewardsServiceError::Internal("ip lookup failed".into())), ApiError::Internal("ip lookup failed".to_string()));
    }

    #[test]
    fn test_boxed_upstream_failure_is_logged_not_returned() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(ClientError::Http {
            status: 500,
            body: b"NoMethodError (undefined method '[]' for nil)".to_vec(),
        });

        let (status, message, detail) = ApiError::from(error).public();

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some("upstream status 500: NoMethodError (undefined method '[]' for nil)"));
    }

    #[test]
    fn test_database_error_is_logged_not_returned() {
        let raw = "duplicate key value violates unique constraint \"devices_device_id_key\"";
        let (status, message, detail) = ApiError::from(DatabaseError::Error(raw.to_string())).public();

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some(raw));
    }

    #[test]
    fn test_unknown_boxed_error_is_logged_not_returned() {
        let error: Box<dyn std::error::Error + Send + Sync> = "connection refused".into();
        let (status, message, detail) = ApiError::from(error).public();

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(message, INTERNAL_ERROR_MESSAGE);
        assert_eq!(detail.as_deref(), Some("connection refused"));
    }

    #[test]
    fn test_user_facing_errors_keep_their_message() {
        assert_eq!(ApiError::OkError("Rate limit reached".to_string()).public(), (StatusCode::OK, "Rate limit reached".to_string(), None));
        assert_eq!(ApiError::NotFound("Device not found".to_string()).public(), (StatusCode::NOT_FOUND, "Device not found".to_string(), None));
    }

    #[test]
    fn test_request_error_maps_to_forbidden() {
        assert_eq!(ApiError::from(RequestError::Forbidden), ApiError::Forbidden);
    }

    #[test]
    fn test_fiat_service_error_mapping() {
        assert_eq!(ApiError::from(FiatServiceError::Request(RequestError::LimitReached)), ApiError::OkError("Rate limit reached".to_string()));
        assert_eq!(ApiError::from(FiatServiceError::Request(RequestError::Forbidden)), ApiError::Forbidden);
        assert_eq!(ApiError::from(FiatServiceError::Quote(FiatQuoteError::InvalidWebhook)), ApiError::BadRequest("Invalid webhook payload".to_string()));
        assert_eq!(ApiError::from(FiatServiceError::Internal("provider down".into())), ApiError::Internal("provider down".to_string()));
    }
}
