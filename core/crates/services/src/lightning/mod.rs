use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use gem_client::{Client, ReqwestClient};
use jsonwebtoken::dangerous::insecure_decode_claims;
use primitives::time::unix_timestamp;
use primitives::{AccessTokenCacher, LightningToken};
use serde::Deserialize;

const TOKEN_PATH: &str = "/api/jwt";

#[derive(Deserialize)]
struct TokenResponse {
    token: String,
}

#[derive(Deserialize)]
struct TokenClaims {
    exp: u64,
}

pub struct LightningTokenClient {
    client: ReqwestClient,
    key: String,
    cacher: Arc<dyn AccessTokenCacher>,
}

impl LightningTokenClient {
    pub fn new(client: ReqwestClient, key: String, cacher: Arc<dyn AccessTokenCacher>) -> Self {
        Self { client, key, cacher }
    }

    pub async fn token(&self) -> Result<LightningToken, Box<dyn Error + Send + Sync>> {
        let token = self.cacher.get_or_refresh(Box::pin(self.refresh())).await?;
        let expires_at = expires_at(&token)?;
        Ok(LightningToken { token, expires_at })
    }

    async fn refresh(&self) -> Result<(String, Duration), Box<dyn Error + Send + Sync>> {
        let headers = HashMap::from([("Authorization".to_string(), format!("Bearer {}", self.key))]);
        let response: TokenResponse = self.client.get_with(TOKEN_PATH, headers).await?;
        let ttl = cache_ttl(expires_at(&response.token)?, unix_timestamp());
        Ok((response.token, ttl))
    }
}

fn expires_at(token: &str) -> Result<u64, Box<dyn Error + Send + Sync>> {
    Ok(insecure_decode_claims::<TokenClaims>(token)?.exp)
}

fn cache_ttl(expires_at: u64, now: u64) -> Duration {
    Duration::from_secs(expires_at.saturating_sub(now) / 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJhdWQiOlsic3Bhcmstc28iLCJzcGFyay1zc3AiXSwiZXhwIjoxNzk1MjAxODYyLCJpYXQiOjE3OTQ1OTcwNjIsImlzcyI6ImJyZWV6LnRlY2hub2xvZ3kifQ.c2lnbmF0dXJl";

    #[test]
    fn test_expires_at() {
        assert_eq!(expires_at(TOKEN).unwrap(), 1795201862);
        assert!(expires_at("token").is_err());
    }

    #[test]
    fn test_cache_ttl() {
        assert_eq!(cache_ttl(1795201862, 1794597062), Duration::from_secs(302400));
        assert_eq!(cache_ttl(100, 200), Duration::ZERO);
    }
}
