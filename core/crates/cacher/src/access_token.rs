use std::{error::Error, time::Duration};

use gem_tracing::warn_with_fields;
use primitives::{AccessTokenCacher, AccessTokenFuture};
use tokio::sync::Semaphore;

use crate::CacherClient;

const EXPIRATION_SAFETY_MARGIN: Duration = Duration::from_secs(60);

pub struct AccessTokenCacherClient {
    cacher: CacherClient,
    key: String,
    refresh: Semaphore,
}

impl AccessTokenCacherClient {
    pub fn new(cacher: CacherClient, provider: &str) -> Self {
        Self {
            cacher,
            key: format!("access_token:{provider}"),
            refresh: Semaphore::new(1),
        }
    }

    async fn cached_access_token(&self) -> Option<String> {
        match self.get().await {
            Ok(access_token) => access_token,
            Err(error) => {
                warn_with_fields!("access token cache read failed", key = self.key.as_str(), error = error.as_ref());
                None
            }
        }
    }

    async fn get(&self) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        self.cacher.get_value_optional::<String>(&self.key).await
    }

    async fn set(&self, access_token: &str, provider_ttl: Duration) -> Result<(), Box<dyn Error + Send + Sync>> {
        let ttl = provider_ttl.saturating_sub(EXPIRATION_SAFETY_MARGIN);
        if ttl.is_zero() {
            return Ok(());
        }

        self.cacher.set_value_with_ttl(&self.key, serde_json::to_string(access_token)?, ttl.as_secs()).await
    }
}

impl AccessTokenCacher for AccessTokenCacherClient {
    fn get_or_refresh<'a>(&'a self, refresh: AccessTokenFuture<'a, (String, Duration)>) -> AccessTokenFuture<'a, String> {
        Box::pin(async move {
            if let Some(access_token) = self.cached_access_token().await {
                return Ok(access_token);
            }

            let _permit = self.refresh.acquire().await?;
            if let Some(access_token) = self.cached_access_token().await {
                return Ok(access_token);
            }

            let (access_token, ttl) = refresh.await?;
            if let Err(error) = self.set(&access_token, ttl).await {
                warn_with_fields!("access token cache write failed", key = self.key.as_str(), error = error.as_ref());
            }
            Ok(access_token)
        })
    }
}
