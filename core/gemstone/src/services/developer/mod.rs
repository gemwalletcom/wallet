pub mod sample;
pub mod store;

use std::sync::Arc;

use primitives::{BannerState, Deeplink, PlatformStore, WalletId};

use crate::services::device::GemDevicePlatform;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::transaction_state::GemTransactionStateStore;
use crate::services::transactions::GemTransactionStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;

pub use store::GemDeveloperStore;

#[derive(uniffi::Object)]
pub struct GemDeveloperService {
    platform: Arc<dyn GemDevicePlatform>,
    preferences: Arc<GemPreferencesService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    transaction_state: Arc<dyn GemTransactionStateStore>,
    transactions: Arc<dyn GemTransactionStore>,
    perpetual: Arc<GemPerpetualService>,
    store: Arc<dyn GemDeveloperStore>,
}

#[uniffi::export]
impl GemDeveloperService {
    #[uniffi::constructor]
    pub fn new(
        platform: Arc<dyn GemDevicePlatform>,
        preferences: Arc<GemPreferencesService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        transaction_state: Arc<dyn GemTransactionStateStore>,
        transactions: Arc<dyn GemTransactionStore>,
        perpetual: Arc<GemPerpetualService>,
        store: Arc<dyn GemDeveloperStore>,
    ) -> Self {
        Self {
            platform,
            preferences,
            wallet_preferences,
            transaction_state,
            transactions,
            perpetual,
            store,
        }
    }

    pub async fn device_id(&self) -> Result<String, GemServiceError> {
        self.platform.device_id().await
    }

    pub async fn push_token(&self) -> Result<String, GemServiceError> {
        self.platform.push_token().await
    }

    pub async fn platform_store(&self) -> Result<PlatformStore, GemServiceError> {
        Ok(self.platform.device_info().await?.platform_store)
    }

    pub async fn clear_pending_transactions(&self) -> Result<(), GemServiceError> {
        for pending in self.transaction_state.get_pending_transactions().await? {
            self.transaction_state.delete_transaction(pending.wallet.id, pending.transaction.id).await?;
        }
        Ok(())
    }

    pub fn reset_transactions_timestamp(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.wallet_preferences.reset_transactions_timestamp(wallet_id)
    }

    pub fn delete_wallet_preferences(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.wallet_preferences.delete_preferences(wallet_id)
    }

    pub fn clear_preferences(&self) -> Result<(), GemServiceError> {
        self.preferences.clear()
    }

    pub async fn clear_perpetual_markets(&self) -> Result<(), GemServiceError> {
        self.perpetual.clear_markets().await
    }

    pub async fn clear_transactions(&self) -> Result<(), GemServiceError> {
        self.store.clear_transactions().await
    }

    pub async fn clear_assets(&self) -> Result<(), GemServiceError> {
        self.store.clear_tokens().await
    }

    pub async fn clear_delegations(&self) -> Result<(), GemServiceError> {
        self.store.clear_delegations().await
    }

    pub async fn clear_validators(&self) -> Result<(), GemServiceError> {
        self.store.clear_validators().await
    }

    pub async fn clear_prices(&self) -> Result<(), GemServiceError> {
        self.store.clear_prices().await
    }

    pub async fn clear_banners(&self) -> Result<(), GemServiceError> {
        self.store.clear_banners().await
    }

    pub async fn activate_cancelled_banners(&self) -> Result<(), GemServiceError> {
        self.store.update_banner_states(BannerState::Cancelled, BannerState::Active).await
    }

    pub async fn add_sample_transactions(&self, wallet_id: WalletId) -> Result<(), GemServiceError> {
        self.transactions.save_transactions(wallet_id, sample::sample_transactions()).await
    }

    pub fn deeplink_url(&self, deeplink: Deeplink) -> String {
        deeplink.to_gem_url()
    }
}
