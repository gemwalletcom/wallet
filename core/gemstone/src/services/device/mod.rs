pub mod platform;
pub mod rules;
pub mod signer;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::Device;

pub use platform::{GemDeviceInfo, GemDevicePlatform};
pub use signer::GemDeviceRequestSigner;
pub use store::GemDeviceStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::preferences::GemPreferencesService;
use crate::services::subscription::GemSubscriptionService;
use crate::services::wallet::GemWalletStore;
use futures::lock::Mutex;

#[derive(uniffi::Object)]
pub struct GemDeviceService {
    api: Arc<GemDeviceApiClient>,
    subscriptions: Arc<GemSubscriptionService>,
    wallet_store: Arc<dyn GemWalletStore>,
    store: Arc<dyn GemDeviceStore>,
    platform: Arc<dyn GemDevicePlatform>,
    preferences: Arc<GemPreferencesService>,
    sync_lock: Mutex<()>,
}

#[uniffi::export]
impl GemDeviceService {
    #[uniffi::constructor]
    pub fn new(
        api: Arc<GemDeviceApiClient>,
        subscriptions: Arc<GemSubscriptionService>,
        wallet_store: Arc<dyn GemWalletStore>,
        store: Arc<dyn GemDeviceStore>,
        platform: Arc<dyn GemDevicePlatform>,
        preferences: Arc<GemPreferencesService>,
    ) -> Self {
        Self {
            api,
            subscriptions,
            wallet_store,
            store,
            platform,
            preferences,
            sync_lock: Mutex::new(()),
        }
    }

    pub async fn is_registered(&self) -> Result<bool, GemServiceError> {
        self.store.is_registered().await
    }

    pub async fn synchronize(&self) -> Result<Device, GemServiceError> {
        let _guard = self.sync_lock.lock().await;
        self.sync(self.current_device().await?).await
    }

    pub async fn synchronize_if_needed(&self) -> Result<(), GemServiceError> {
        let _guard = self.sync_lock.lock().await;
        let device = self.current_device().await?;
        if self.needs_sync(device.clone()).await? {
            self.sync(device).await?;
        }
        Ok(())
    }
}

impl GemDeviceService {
    async fn current_device(&self) -> Result<Device, GemServiceError> {
        let info = self.platform.device_info().await?;
        Ok(Device {
            id: self.platform.device_id().await?,
            platform: info.platform,
            platform_store: info.platform_store,
            os: info.os,
            model: info.model,
            token: self.platform.push_token().await?,
            locale: info.locale,
            version: info.version,
            currency: self.platform.currency().await?,
            is_push_enabled: self.platform.is_push_enabled().await?,
            is_price_alerts_enabled: Some(self.preferences.is_price_alerts_enabled()?),
            subscriptions_version: 0,
        })
    }

    async fn needs_sync(&self, device: Device) -> Result<bool, GemServiceError> {
        if !self.store.is_registered().await? {
            return Ok(true);
        }
        let Some(pushed) = self.store.get_pushed_device().await? else {
            return Ok(true);
        };
        let local = Device {
            subscriptions_version: self.store.get_subscriptions_version().await?,
            ..device
        };
        if rules::device_changed(&pushed, &local) {
            return Ok(true);
        }
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets()?);
        Ok(self.store.get_pushed_subscriptions().await? != Some(signature))
    }

    async fn sync(&self, device: Device) -> Result<Device, GemServiceError> {
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets()?);
        let mut version = self.store.get_subscriptions_version().await?;
        let remote = self.get_or_create(&device).await?;

        let signature_changed = self.store.get_pushed_subscriptions().await? != Some(signature.clone());
        if signature_changed || remote.subscriptions_version != version {
            self.subscriptions.sync().await?;
            if signature_changed {
                version += 1;
                self.store.set_subscriptions_version(version).await?;
            }
        }

        let local = Device {
            subscriptions_version: version,
            ..device
        };
        let synced = if rules::device_changed(&remote, &local) {
            self.api.client.update_device(local.clone()).await.map_err(GemApiError::from)?
        } else {
            remote
        };
        self.store.set_pushed_device(local).await?;
        self.store.set_pushed_subscriptions(signature).await?;
        Ok(synced)
    }
}

impl GemDeviceService {
    async fn get_or_create(&self, device: &Device) -> Result<Device, GemServiceError> {
        let registered = self.store.is_registered().await? || self.api.client.is_device_registered().await.map_err(GemApiError::from)?;
        if registered {
            match self.api.client.get_device().await.map_err(GemApiError::from)? {
                Some(remote) => {
                    self.store.set_registered(true).await?;
                    return Ok(remote);
                }
                None => self.store.set_registered(false).await?,
            }
        }
        let added = self
            .api
            .client
            .add_device(Device {
                subscriptions_version: 0,
                ..device.clone()
            })
            .await
            .map_err(GemApiError::from)?;
        self.store.set_registered(true).await?;
        Ok(added)
    }
}
