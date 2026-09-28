use async_trait::async_trait;
use primitives::{RecentActivityType, WalletId};

use crate::services::error::GemServiceError;
use crate::services::transfer::{GemRecentActivity, GemRecentActivityScope};

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemRecentActivityStore: Send + Sync {
    async fn add(&self, activity: GemRecentActivity, wallet_id: WalletId) -> Result<(), GemServiceError>;
    async fn clear(&self, scope: GemRecentActivityScope, types: Vec<RecentActivityType>) -> Result<(), GemServiceError>;
}
