pub mod error;
pub mod model;
pub mod rules;
pub mod store;

use std::sync::Arc;

use primitives::{AssetId, BannerEvent, BannerState};

pub use error::GemBannerError;
pub use model::{GemBannerContext, GemBannerKey};
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

    pub async fn active_events(&self, wallet_id: Option<String>, asset_id: Option<AssetId>, context: GemBannerContext) -> Result<Vec<BannerEvent>, GemBannerError> {
        let mut active = Vec::new();
        for event in rules::suggested_events(&context) {
            let key = GemBannerKey {
                wallet_id: wallet_id.clone(),
                asset_id: asset_id.clone(),
                chain: None,
                event,
            };
            let state = self.store.get_state(key).await?.unwrap_or_else(|| rules::default_state(event));
            if rules::is_visible(state) {
                active.push(event);
            }
        }
        Ok(active)
    }

    pub async fn close(&self, key: GemBannerKey) -> Result<(), GemBannerError> {
        self.store.set_state(key, BannerState::Cancelled).await
    }

    pub fn closes_on_action(&self, event: BannerEvent) -> bool {
        rules::closes_on_action(event)
    }
}
