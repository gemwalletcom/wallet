use axum::response::{IntoResponse, Response};
use fiat::error::FiatQuoteError;
use gem_auth::JwtError;
use gem_client::ClientError;
use http::StatusCode;
use http_server::{ErrorBody, INTERNAL_ERROR_MESSAGE, status_message};
use localizer::LanguageLocalizer;
use primitives::RequestError;
use services::fiat::FiatServiceError;
use services::rewards::RewardsServiceError;
use services::{CacheError, DatabaseError};

pub const DEFAULT_LANGUAGE: &str = "en";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
    pub detail: Option<String>,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
            detail: None,
        }
    }

    pub fn from_status(status: StatusCode) -> Self {
        Self::new(status, status_message(status))
    }

    pub fn ok(message: impl Into<String>) -> Self {
        Self::new(StatusCode::OK, message)
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn payload_too_large(message: impl Into<String>) -> Self {
        Self::new(StatusCode::PAYLOAD_TOO_LARGE, message)
    }

    pub fn unsupported_media_type(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNSUPPORTED_MEDIA_TYPE, message)
    }

    pub fn unprocessable(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    pub fn internal(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: INTERNAL_ERROR_MESSAGE.to_string(),
            detail: Some(detail.into()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut response = ErrorBody::new(self.status, self.message.clone()).into_response();
        response.extensions_mut().insert(self);
        response
    }
}

pub fn fiat_error(error: FiatServiceError, language: &str) -> ApiError {
    let localizer = LanguageLocalizer::new_with_language(language);
    match error {
        FiatServiceError::Request(RequestError::LimitReached) => ApiError::ok(localizer.fiat_error_limit_reached()),
        FiatServiceError::Request(RequestError::Forbidden) => ApiError::bad_request(localizer.fiat_error_quote_unavailable()),
        FiatServiceError::Quote(FiatQuoteError::RegionUnavailable) => ApiError::ok(localizer.fiat_error_region_unavailable()),
        FiatServiceError::Quote(FiatQuoteError::ProviderRejected) => ApiError::ok(localizer.fiat_error_quote_unavailable()),
        FiatServiceError::Quote(FiatQuoteError::MinimumAmount(_) | FiatQuoteError::UnsupportedState(_) | FiatQuoteError::InvalidRequest(_) | FiatQuoteError::InvalidWebhook) => ApiError::bad_request(localizer.errors_generic()),
        FiatServiceError::Storage(error) => error.into(),
        FiatServiceError::Internal(error) => ApiError::internal(error.to_string()),
    }
}

impl From<FiatServiceError> for ApiError {
    fn from(error: FiatServiceError) -> Self {
        fiat_error(error, DEFAULT_LANGUAGE)
    }
}

impl From<CacheError> for ApiError {
    fn from(error: CacheError) -> Self {
        match error {
            CacheError::NotFound { .. } | CacheError::ResourceNotFound(_) => ApiError::not_found(error.to_string()),
            CacheError::KeyNotFound(_) => ApiError::internal("Unexpected cache miss"),
        }
    }
}

impl From<JwtError> for ApiError {
    fn from(error: JwtError) -> Self {
        ApiError::internal(error.to_string())
    }
}

impl From<DatabaseError> for ApiError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound { .. } => ApiError::not_found(error.to_string()),
            DatabaseError::ConnectionPool => ApiError::internal(error.to_string()),
            DatabaseError::Error(message) => ApiError::internal(message),
        }
    }
}

impl From<swapper::SwapperError> for ApiError {
    fn from(error: swapper::SwapperError) -> Self {
        ApiError::internal(error.to_string())
    }
}

impl From<RewardsServiceError> for ApiError {
    fn from(error: RewardsServiceError) -> Self {
        match error {
            RewardsServiceError::Rejected(error) => ApiError::ok(error.to_string()),
            RewardsServiceError::Storage(error) => error.into(),
            RewardsServiceError::Internal(error) => ApiError::internal(error.to_string()),
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
                return ApiError::internal(format!("upstream status {status}: {}", String::from_utf8_lossy(body)));
            }
            match current_error.source() {
                Some(source) => current_error = source,
                None => break,
            }
        }

        ApiError::internal(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::{ApiError, INTERNAL_ERROR_MESSAGE, fiat_error};
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
        assert_eq!(fiat_error(FiatServiceError::Quote(FiatQuoteError::RegionUnavailable), "en"), ApiError::ok("Not available in your region."));
        assert_eq!(
            fiat_error(FiatServiceError::Request(RequestError::LimitReached), "de"),
            ApiError::ok("Zu viele Angebotsanfragen. Bitte versuchen Sie es in ein paar Minuten erneut.")
        );
        assert_eq!(
            fiat_error(FiatServiceError::Request(RequestError::Forbidden), "en"),
            ApiError::bad_request("This quote is no longer available. Please try again."),
            "a quote that expired says so instead of a bare Forbidden"
        );
        assert_eq!(
            fiat_error(FiatServiceError::Quote(FiatQuoteError::InvalidRequest("Missing network".to_string())), "en"),
            ApiError::bad_request("An unexpected error occurred. Please try again later.")
        );
        assert_eq!(fiat_error(FiatServiceError::Internal("connection refused".into()), "en"), ApiError::internal("connection refused"));
        assert_eq!(
            fiat_error(FiatServiceError::Quote(FiatQuoteError::ProviderRejected), "en"),
            ApiError::ok("This quote is no longer available. Please try again."),
            "a provider rejecting the quote is an outcome the app shows, not a server error"
        );
        assert_eq!(fiat_error(FiatServiceError::Storage(DatabaseError::not_found("Asset", "btc")), "en"), ApiError::not_found("Asset btc not found"));
        assert_eq!(
            ApiError::from(FiatServiceError::Request(RequestError::LimitReached)),
            ApiError::ok("Too many quote requests. Please try again in a few minutes.")
        );
    }

    #[test]
    fn test_cache_errors_map_to_public_statuses() {
        assert_eq!(ApiError::from(CacheError::not_found("FiatQuote", "abc")), ApiError::not_found("FiatQuote abc not found"));
        assert_eq!(ApiError::from(CacheError::KeyNotFound("fiat:quote:abc".to_string())), ApiError::internal("Unexpected cache miss"));
    }

    #[test]
    fn test_boxed_database_not_found_hides_internal_lookup() {
        let error: Box<dyn std::error::Error + Send + Sync> = Box::new(DatabaseError::not_found_internal("Device", "1"));
        assert_eq!(ApiError::from(error), ApiError::not_found("Device not found"));
    }

    #[test]
    fn test_rewards_service_error_mapping() {
        let rejected = RewardsServiceError::Rejected(RewardsError::Username("Daily username creation limit has been reached".to_string()));
        assert_eq!(ApiError::from(rejected), ApiError::ok("Daily username creation limit has been reached"));
        assert_eq!(ApiError::from(RewardsServiceError::Storage(DatabaseError::not_found_internal("Rewards", "1"))), ApiError::not_found("Rewards not found"));
        assert_eq!(ApiError::from(RewardsServiceError::Internal("ip lookup failed".into())), ApiError::internal("ip lookup failed"));
    }

    #[test]
    fn test_failures_are_logged_not_returned() {
        let upstream: Box<dyn std::error::Error + Send + Sync> = Box::new(ClientError::Http {
            status: 500,
            body: b"NoMethodError (undefined method '[]' for nil)".to_vec(),
        });
        let unknown: Box<dyn std::error::Error + Send + Sync> = "connection refused".into();
        let raw = "duplicate key value violates unique constraint \"devices_device_id_key\"";

        for (error, detail) in [
            (ApiError::from(upstream), "upstream status 500: NoMethodError (undefined method '[]' for nil)"),
            (ApiError::from(unknown), "connection refused"),
            (ApiError::from(DatabaseError::Error(raw.to_string())), raw),
        ] {
            assert_eq!(error.status, StatusCode::INTERNAL_SERVER_ERROR);
            assert_eq!(error.message, INTERNAL_ERROR_MESSAGE);
            assert_eq!(error.detail.as_deref(), Some(detail));
        }
    }

    #[test]
    fn test_user_facing_errors_keep_their_message_and_status() {
        assert_eq!(ApiError::ok("Rate limit reached"), ApiError::new(StatusCode::OK, "Rate limit reached"));
        assert_eq!(ApiError::not_found("Device not found").status, StatusCode::NOT_FOUND);
        assert_eq!(ApiError::from_status(StatusCode::GATEWAY_TIMEOUT), ApiError::new(StatusCode::GATEWAY_TIMEOUT, "504 Gateway Timeout"));
        assert!(ApiError::bad_request("x").detail.is_none());
    }
}
