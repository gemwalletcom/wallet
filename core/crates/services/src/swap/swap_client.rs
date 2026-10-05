use std::error::Error;

use primitives::FiatAssets;
use std::sync::Arc;

use super::repository::Repository;

#[derive(Clone)]
pub struct SwapClient {
    repository: Arc<dyn Repository>,
}

impl SwapClient {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        Ok(FiatAssets::new(self.repository.swap_asset_ids().await?))
    }
}
