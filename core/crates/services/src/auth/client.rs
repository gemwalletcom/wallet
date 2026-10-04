use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use cacher::{CacheKey, CacherClient};
use chrono::Utc;
use primitives::AuthNonce;
use uuid::Uuid;

#[async_trait]
pub trait AuthNonceCacher: Send + Sync {
    async fn add_nonce(&self, device_id: &str, nonce: &AuthNonce) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn take_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl AuthNonceCacher for CacherClient {
    async fn add_nonce(&self, device_id: &str, nonce: &AuthNonce) -> Result<(), Box<dyn Error + Send + Sync>> {
        let cache_key = CacheKey::AuthNonce(device_id, &nonce.nonce);
        self.set_value_with_ttl(&cache_key.key(), serde_json::to_string(nonce)?, cache_key.ttl()).await
    }

    async fn take_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        self.get_and_delete_value::<AuthNonce>(&CacheKey::AuthNonce(device_id, nonce).key()).await
    }
}

pub struct AuthClient {
    nonces: Arc<dyn AuthNonceCacher>,
}

impl AuthClient {
    pub fn new(nonces: Arc<dyn AuthNonceCacher>) -> Self {
        Self { nonces }
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
