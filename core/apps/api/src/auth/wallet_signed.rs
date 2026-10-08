use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts, State};
use gem_auth::verify_auth_signature;
use http::request::Parts;
use primitives::{AuthMessage, AuthenticatedRequest};
use serde::de::DeserializeOwned;
use services::auth::AuthClient;

use crate::auth::device::guard::{json_body, verified};
use crate::error::ApiError;

pub struct WalletSigned<T> {
    pub address: String,
    pub data: T,
}

impl<T, S> FromRequestParts<S> for WalletSigned<T>
where
    T: DeserializeOwned + Send,
    S: Send + Sync,
    Arc<AuthClient>: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let body: AuthenticatedRequest<T> = json_body(parts)?;
        let request = verified(parts)?;
        if body.auth.device_id != request.device_id {
            return Err(ApiError::BadRequest("Wallet signature device mismatch".to_string()));
        }
        let State(auth_client): State<Arc<AuthClient>> = State::from_request_parts(parts, state).await.map_err(|_| ApiError::Internal("Auth client not available".to_string()))?;
        let auth_nonce = auth_client.consume_auth_nonce(&body.auth.device_id, &body.auth.nonce).await.map_err(|_| ApiError::Unauthorized("Invalid nonce".to_string()))?;
        let auth_message = AuthMessage {
            chain: body.auth.chain,
            address: body.auth.address.clone(),
            auth_nonce,
        };
        if !verify_auth_signature(&auth_message, &body.auth.signature) {
            return Err(ApiError::Unauthorized("Invalid signature".to_string()));
        }
        Ok(Self { address: body.auth.address, data: body.data })
    }
}
