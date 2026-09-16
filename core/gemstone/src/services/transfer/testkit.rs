use std::sync::Mutex;

use async_trait::async_trait;
use primitives::{RecentActivityType, TransactionInputType, WalletId};

use super::{GemRecentActivity, GemRecentActivityStore, GemRecipient, GemTransferData};
use crate::services::error::GemServiceError;

impl GemTransferData {
    pub fn mock(input_type: TransactionInputType) -> Self {
        Self {
            input_type,
            recipient: GemRecipient::address("recipient".to_string()),
            value: 0.into(),
            use_max_amount: false,
        }
    }
}

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
