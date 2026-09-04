use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{RecentActivityType, WalletId};

use super::{GemRecentActivity, GemRecentActivityStore};
use crate::services::error::GemServiceError;

#[derive(Default)]
pub struct MemoryRecentActivityStore {
    pub added: Mutex<Vec<(GemRecentActivity, WalletId)>>,
}

#[async_trait]
impl GemRecentActivityStore for MemoryRecentActivityStore {
    async fn add(&self, activity: GemRecentActivity, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.added.lock().unwrap().push((activity, wallet_id));
        Ok(())
    }
    async fn clear(&self, _wallet_id: WalletId, _types: Vec<RecentActivityType>) -> Result<(), GemServiceError> {
        self.added.lock().unwrap().clear();
        Ok(())
    }
}
