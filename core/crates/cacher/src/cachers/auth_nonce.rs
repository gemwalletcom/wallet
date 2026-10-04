use std::error::Error;

use async_trait::async_trait;
use primitives::AuthNonce;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait AuthNonceCacher: Send + Sync {
    async fn add_nonce(&self, device_id: &str, nonce: &AuthNonce) -> Result<(), Box<dyn Error + Send + Sync>>;
    async fn take_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl AuthNonceCacher for CacherClient {
    async fn add_nonce(&self, device_id: &str, nonce: &AuthNonce) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(CacheKey::AuthNonce(device_id, &nonce.nonce), nonce).await
    }

    async fn take_nonce(&self, device_id: &str, nonce: &str) -> Result<AuthNonce, Box<dyn Error + Send + Sync>> {
        self.take(CacheKey::AuthNonce(device_id, nonce)).await
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires REDIS_URL pointing at a disposable Redis instance"]
    async fn test_take_nonce_is_atomic_under_concurrency() {
        let redis_url = std::env::var("REDIS_URL").unwrap();
        let client = CacherClient::new(&redis_url).await.unwrap();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let device_id = format!("test-device-{}-{timestamp}", std::process::id());
        let nonce = AuthNonce {
            nonce: "single-use".to_string(),
            timestamp: 0,
        };

        client.add_nonce(&device_id, &nonce).await.unwrap();

        let handles = (0..32)
            .map(|_| {
                let client = client.clone();
                let device_id = device_id.clone();
                tokio::spawn(async move { client.take_nonce(&device_id, "single-use").await.ok() })
            })
            .collect::<Vec<_>>();

        let mut consumed_nonces = Vec::new();
        for handle in handles {
            if let Some(consumed_nonce) = handle.await.unwrap() {
                consumed_nonces.push(consumed_nonce.nonce);
            }
        }

        assert_eq!(consumed_nonces, vec!["single-use".to_string()]);
        assert!(client.take_nonce(&device_id, "single-use").await.is_err());
    }
}
