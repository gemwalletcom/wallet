use std::error::Error;

use primitives::Chain;

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn add_pending(&self, chain: Chain, hash: String, expires_at: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.add_to_sorted_set(pending_key(&chain), &[(hash, expires_at)]).await?;
        Ok(())
    }

    pub async fn pending(&self, chain: Chain) -> Result<Vec<(String, f64)>, Box<dyn Error + Send + Sync>> {
        self.sorted_set(pending_key(&chain)).await
    }

    pub async fn remove_pending(&self, chain: Chain, hash: &str) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.remove_from_sorted_set(pending_key(&chain), &[hash.to_string()]).await
    }

    pub async fn pending_count(&self, chain: Chain) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.sorted_set_count(pending_key(&chain)).await
    }
}

fn pending_key(chain: &Chain) -> CacheKey<'_> {
    CacheKey::PendingTransactions(chain.as_ref())
}
