use crate::services::error::GemServiceError;
use async_trait::async_trait;
use primitives::BannerState;

#[uniffi::export(rust, foreign)]
#[async_trait]
pub trait GemDeveloperStore: Send + Sync {
    async fn clear_transactions(&self) -> Result<(), GemServiceError>;
    async fn clear_tokens(&self) -> Result<(), GemServiceError>;
    async fn clear_delegations(&self) -> Result<(), GemServiceError>;
    async fn clear_validators(&self) -> Result<(), GemServiceError>;
    async fn clear_prices(&self) -> Result<(), GemServiceError>;
    async fn clear_banners(&self) -> Result<(), GemServiceError>;
    async fn update_banner_states(&self, from: BannerState, to: BannerState) -> Result<(), GemServiceError>;
}
