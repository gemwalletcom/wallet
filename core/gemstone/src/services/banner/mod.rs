pub mod model;
pub mod permissions;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{BannerState, Wallet};

pub use model::{GemBannerAction, GemBannerContext, GemBannerItem, GemBannerKey};
pub use permissions::GemNotificationPermissions;
pub use store::GemBannerStore;

#[derive(uniffi::Object)]
pub struct GemBannerService {
    store: Arc<dyn GemBannerStore>,
}

#[uniffi::export]
impl GemBannerService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemBannerStore>) -> Self {
        Self { store }
    }

    pub async fn setup(&self) -> Result<(), GemServiceError> {
        self.store.add_banners(rules::setup_keys(), BannerState::Active).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        self.store.add_banners(rules::wallet_setup_keys(&wallet), BannerState::Active).await
    }

    pub async fn apply_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        match action {
            GemBannerAction::Close => self.close(key).await,
            GemBannerAction::Event { .. } | GemBannerAction::Button => Ok(()),
        }
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.store.set_state(key, BannerState::Cancelled).await
    }

    pub fn visible_banners(&self, stored: Vec<GemBannerItem>, context: GemBannerContext) -> Vec<GemBannerItem> {
        rules::visible_banners(stored, &context)
    }
}
