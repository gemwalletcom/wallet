use std::error::Error;

use primitives::ScanType;

use crate::{CacheKey, CacherClient};

pub struct SafeScanTarget {
    pub scan_type: ScanType,
    pub target: String,
    pub ttl: u64,
}

impl SafeScanTarget {
    fn cache_key(&self) -> CacheKey<'_> {
        CacheKey::ScanSafe(self.scan_type.as_ref(), &self.target, self.ttl)
    }
}

impl CacherClient {
    pub async fn is_safe(&self, target: &SafeScanTarget) -> Result<bool, Box<dyn Error + Send + Sync>> {
        Ok(self.get::<bool>(target.cache_key()).await?.is_some())
    }

    pub async fn add_safe(&self, targets: &[&SafeScanTarget]) -> Result<(), Box<dyn Error + Send + Sync>> {
        let entries = targets.iter().map(|target| (target.cache_key(), &true)).collect::<Vec<_>>();
        self.set_many(&entries).await?;
        Ok(())
    }
}
