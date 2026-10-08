use std::error::Error;

use async_trait::async_trait;

use crate::{CacheKey, CacherClient};

#[async_trait]
pub trait DeviceSignatureCacher: Send + Sync {
    async fn remember_signature(&self, signature: &str) -> Result<bool, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
impl DeviceSignatureCacher for CacherClient {
    async fn remember_signature(&self, signature: &str) -> Result<bool, Box<dyn Error + Send + Sync>> {
        self.set_if_absent(CacheKey::DeviceRequestSignature(signature)).await
    }
}
