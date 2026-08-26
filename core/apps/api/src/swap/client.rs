use std::error::Error;

use primitives::FiatAssets;
use storage::{AssetsRepository, Database};

#[derive(Clone)]
pub struct SwapClient {
    database: Database,
}

impl SwapClient {
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    pub async fn get_swap_assets(&self) -> Result<FiatAssets, Box<dyn Error + Send + Sync>> {
        Ok(FiatAssets::new(self.database.assets()?.get_swap_assets()?))
    }
}
