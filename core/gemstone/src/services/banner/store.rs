use async_trait::async_trait;
use primitives::BannerState;

use super::error::GemBannerError;
use super::model::GemBannerKey;

#[uniffi::export(with_foreign)]
#[async_trait]
pub trait GemBannerStore: Send + Sync {
    async fn get_state(&self, key: GemBannerKey) -> Result<Option<BannerState>, GemBannerError>;
    async fn set_state(&self, key: GemBannerKey, state: BannerState) -> Result<(), GemBannerError>;
}
