pub mod model;
pub mod permissions;
pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{AssetId, BannerEvent, BannerState, Wallet, WalletId};

pub use model::{GemBannerAction, GemBannerContext, GemBannerItem, GemBannerKey};
pub use permissions::GemNotificationPermissions;
pub use store::GemBannerStore;

#[derive(uniffi::Object)]
pub struct GemBannerService {
    store: Arc<dyn GemBannerStore>,
    permissions: Arc<dyn GemNotificationPermissions>,
}

#[uniffi::export]
impl GemBannerService {
    #[uniffi::constructor]
    pub fn new(store: Arc<dyn GemBannerStore>, permissions: Arc<dyn GemNotificationPermissions>) -> Self {
        Self { store, permissions }
    }

    pub async fn setup(&self) -> Result<(), GemServiceError> {
        self.store.add_banners(rules::setup_keys(), BannerState::Active).await
    }

    pub async fn setup_wallet(&self, wallet: Wallet) -> Result<(), GemServiceError> {
        self.store.add_banners(rules::wallet_setup_keys(&wallet), BannerState::Active).await
    }

    pub async fn apply_action(&self, key: GemBannerKey, action: GemBannerAction) -> Result<(), GemServiceError> {
        let closes = match rules::close_decision(&action) {
            rules::BannerClose::Close => true,
            rules::BannerClose::Keep => false,
            rules::BannerClose::AfterPermission => self.permissions.request_permissions_or_open_settings().await?,
        };
        if closes {
            self.close(key).await?;
        }
        Ok(())
    }

    pub async fn active_events(&self, wallet_id: Option<WalletId>, asset_id: Option<AssetId>, context: GemBannerContext) -> Result<Vec<BannerEvent>, GemServiceError> {
        let mut active = Vec::new();
        for event in rules::suggested_events(&context) {
            let key = rules::event_key(wallet_id.clone(), asset_id.clone(), event);
            let state = self.store.get_state(key).await?.unwrap_or_else(|| rules::default_state(event));
            if rules::is_visible(state) {
                active.push(event);
            }
        }
        Ok(active)
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemServiceError> {
        self.store.set_state(key, BannerState::Cancelled).await
    }

    pub fn visible_banners(&self, stored: Vec<GemBannerItem>, context: GemBannerContext) -> Vec<GemBannerItem> {
        rules::visible_banners(stored, &context)
    }
}
