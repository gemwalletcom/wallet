use std::error::Error;
use std::sync::Arc;

use super::repository::Repository;

pub struct DeviceUpdater {
    repository: Arc<dyn Repository>,
}

impl DeviceUpdater {
    pub(crate) fn new(repository: Arc<dyn Repository>) -> Self {
        Self { repository }
    }

    pub async fn update(&self) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Ok(self.repository.delete_subscriptions_after_days(120).await?)
    }
}
