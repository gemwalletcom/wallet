use std::sync::Arc;

use primitives::{Deeplink, PlatformStore, WalletId};

use crate::services::device::GemDevicePlatform;
use crate::services::error::GemServiceError;
use crate::services::perpetual::GemPerpetualService;
use crate::services::preferences::GemPreferencesService;
use crate::services::transaction_state::GemTransactionStateStore;
use crate::services::wallet_preferences::GemWalletPreferencesService;

#[derive(uniffi::Object)]
pub struct GemDeveloperService {
    platform: Arc<dyn GemDevicePlatform>,
    preferences: Arc<GemPreferencesService>,
    wallet_preferences: Arc<GemWalletPreferencesService>,
    transactions: Arc<dyn GemTransactionStateStore>,
    perpetual: Arc<GemPerpetualService>,
}

#[uniffi::export]
impl GemDeveloperService {
    #[uniffi::constructor]
    pub fn new(
        platform: Arc<dyn GemDevicePlatform>,
        preferences: Arc<GemPreferencesService>,
        wallet_preferences: Arc<GemWalletPreferencesService>,
        transactions: Arc<dyn GemTransactionStateStore>,
        perpetual: Arc<GemPerpetualService>,
    ) -> Self {
        Self {
            platform,
            preferences,
            wallet_preferences,
            transactions,
            perpetual,
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
        for pending in self.transactions.get_pending_transactions().await? {
            self.transactions.delete_transaction(pending.wallet.id, pending.transaction.id).await?;
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

    pub fn deeplink_url(&self, deeplink: Deeplink) -> String {
        deeplink.to_gem_url()
    }
}
