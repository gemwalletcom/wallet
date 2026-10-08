use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts, State};
use gem_auth::{AUTHORIZATION_HEADER, BEARER_PREFIX};
use http::request::Parts;
use primitives::OptionStringExt;
use services::access::{AccessClient, ApiClientScope};

use crate::error::ApiError;

pub struct AdminWrite;
pub struct ChainRead;
pub struct DeviceRead;
pub struct DeviceSubscriptionsRead;
pub struct DeviceTransactionsRead;
pub struct FiatQuotesRead;
pub struct FiatTransactionsRead;

fn bearer_secret(parts: &Parts) -> Result<&str, ApiError> {
    let value = parts
        .headers
        .get(AUTHORIZATION_HEADER)
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| ApiError::Unauthorized("Missing Authorization header".to_string()))?;
    value.strip_prefix(BEARER_PREFIX).non_empty().ok_or_else(|| ApiError::Unauthorized("Invalid authorization format".to_string()))
}

async fn authorize<S>(parts: &mut Parts, state: &S, scope: ApiClientScope) -> Result<(), ApiError>
where
    S: Send + Sync,
    Arc<AccessClient>: FromRef<S>,
{
    let secret = bearer_secret(parts)?.to_string();
    let State(access): State<Arc<AccessClient>> = State::from_request_parts(parts, state).await.map_err(|_| ApiError::Internal("Database not available".to_string()))?;
    let allowed = access.is_api_client_allowed(&secret, scope).await.map_err(|_| ApiError::Internal("Failed to load API client".to_string()))?;
    if !allowed {
        return Err(ApiError::Unauthorized(format!("Invalid API client for scope {}", scope.as_ref())));
    }
    Ok(())
}

macro_rules! permission {
    ($guard:ident, $scope:expr) => {
        impl<S> FromRequestParts<S> for $guard
        where
            S: Send + Sync,
            Arc<AccessClient>: FromRef<S>,
        {
            type Rejection = ApiError;

            async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
                authorize(parts, state, $scope).await.map(|_| $guard)
            }
        }
    };
}

permission!(AdminWrite, ApiClientScope::AdminWrite);
permission!(ChainRead, ApiClientScope::ChainRead);
permission!(DeviceRead, ApiClientScope::DevicesRead);
permission!(DeviceSubscriptionsRead, ApiClientScope::DevicesSubscriptionsRead);
permission!(DeviceTransactionsRead, ApiClientScope::DevicesTransactionsRead);
permission!(FiatQuotesRead, ApiClientScope::FiatQuotesRead);
permission!(FiatTransactionsRead, ApiClientScope::FiatTransactionsRead);
