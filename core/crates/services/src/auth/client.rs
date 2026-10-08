use std::{error::Error, sync::Arc};

use cacher::{AuthNonceCacher, DeviceSignatureCacher};
use chrono::Utc;
use primitives::AuthNonce;
use uuid::Uuid;

pub struct AuthClient {
    nonces: Arc<dyn AuthNonceCacher>,
    signatures: Arc<dyn DeviceSignatureCacher>,
}

impl AuthClient {
    pub fn new(nonces: Arc<dyn AuthNonceCacher>, signatures: Arc<dyn DeviceSignatureCacher>) -> Self {
        Self { nonces, signatures }
    }

    pub async fn remember_request_signature(&self, signature: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.signatures.remember_signature(signature).await
    }

    pub async fn get_nonce(&self, device_id: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        let auth_nonce = AuthNonce {
            nonce: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp() as u32,
        };
        self.nonces.add_nonce(device_id, &auth_nonce).await?;
        Ok(auth_nonce)
    }

    pub async fn consume_auth_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        self.nonces.take_nonce(device_id, nonce).await
    }
}
