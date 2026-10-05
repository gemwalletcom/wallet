use std::error::Error;

use primitives::FiatAssets;
use std::sync::Arc;

use crate::assets::AssetCatalogClient;

#[derive(Clone)]
pub struct SwapClient {
    asset_catalog: Arc<AssetCatalogClient>,
}

impl SwapClient {
    pub(crate) fn new(asset_catalog: Arc<AssetCatalogClient>) -> Self {
        Self { asset_catalog }
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        Ok(self.asset_catalog.get().await?.swap_assets)
    }
}
