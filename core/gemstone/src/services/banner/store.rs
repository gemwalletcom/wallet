use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::BannerState;

use super::model::GemBannerKey;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemBannerStore: Send + Sync {
    async fn get_state(&self, key: GemBannerKey) -> Result<Option<BannerState>, GemServiceError>;
    async fn set_state(&self, key: GemBannerKey, state: BannerState) -> Result<(), GemServiceError>;
}
