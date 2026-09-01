use primitives::WalletId;

use crate::services::error::GemServiceError;
use crate::services::transfer::GemRecentActivity;

#[uniffi::export(rust, foreign)]
pub trait GemRecentActivityStore: Send + Sync {
    fn add(&self, activity: GemRecentActivity, wallet_id: WalletId) -> Result<(), GemServiceError>;
}
