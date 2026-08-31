pub mod model;
pub mod permissions;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Asset, BannerEvent, BannerState, Wallet};

pub use model::{GemBannerAction, GemBannerAmount, GemBannerContent, GemBannerContext, GemBannerDescription, GemBannerIcon, GemBannerItem, GemBannerKey, GemBannerTitle};
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
        match action.is_dismissal() {
            true => self.close(key).await,
            false => Ok(()),
        }
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.store.set_state(key, BannerState::Cancelled).await
    }

    pub fn shows_onboarding(&self, state: BannerState, is_wallet_empty: bool) -> bool {
        rules::shows_onboarding(state, is_wallet_empty)
    }

    pub fn visible_banners(&self, stored: Vec<GemBannerItem>, context: GemBannerContext) -> Vec<GemBannerItem> {
        rules::visible_banners(stored, &context)
    }

    pub fn banner_content(&self, event: BannerEvent, asset: Option<Asset>) -> GemBannerContent {
        rules::banner_content(event, asset.as_ref())
    }
}
