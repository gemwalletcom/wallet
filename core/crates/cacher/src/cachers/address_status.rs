use std::error::Error;

use primitives::{AddressStatus, ChainAddress};

use crate::{CacheKey, CacherClient};

impl CacherClient {
    pub async fn address_statuses(&self, address: &ChainAddress) -> Result<Option<Vec<AddressStatus>>, Box<dyn Error + Send + Sync>> {
        self.get(status_key(address)).await
    }

    pub async fn set_address_statuses(&self, address: &ChainAddress, statuses: &[AddressStatus]) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.set(status_key(address), &statuses).await
    }
}

fn status_key(address: &ChainAddress) -> CacheKey<'_> {
    CacheKey::AddressStatus(address.chain.as_ref(), &address.address)
}
