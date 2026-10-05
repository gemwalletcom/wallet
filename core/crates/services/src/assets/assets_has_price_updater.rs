use std::error::Error;
use std::sync::Arc;

use crate::assets::repository::Repository;

pub struct AssetsHasPriceUpdater {
    repository: Arc<dyn Repository>,
}

impl AssetsHasPriceUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn update(&self) -> Result<(usize, usize), Box<dyn Error + Send + Sync>> {
        Ok(self.repository.update_price_flags().await?)
    }
}
