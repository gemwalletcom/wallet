use std::error::Error;

use cacher::CacherClient;
use gem_crypto::random;
use primitives::{AuthNonce, DateTime, hex};

pub struct AuthClient {
    nonces: CacherClient,
}

impl AuthClient {
    pub fn new(nonces: CacherClient) -> Self {
        Self { nonces }
    }

    pub async fn get_nonce(&self, device_id: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        let auth_nonce = AuthNonce {
            nonce: hex::encode(random::bytes::<16>()?),
            timestamp: DateTime::now().timestamp() as u32,
        };
        self.nonces.add_nonce(device_id, &auth_nonce).await?;
        Ok(auth_nonce)
    }

    pub async fn consume_auth_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        self.nonces.take_nonce(device_id, nonce).await
    }
}
