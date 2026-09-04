pub mod keys;
pub mod platform;
pub mod rules;
pub mod signer;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::{Device, DeviceLocale};

pub use keys::GemDeviceKeyService;
pub use platform::{GemDeviceInfo, GemDevicePlatform};
pub use signer::GemDeviceRequestSigner;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::preferences::GemPreferencesService;
use crate::services::subscription::GemSubscriptionService;
use crate::services::wallet::GemWalletStore;
use futures::lock::Mutex;
use gem_api::WalletRequestPreflight;
use gem_client::ClientError;
use std::sync::Weak;

#[derive(uniffi::Object)]
pub struct GemDeviceService {
    api: Arc<GemDeviceApiClient>,
    subscriptions: Arc<GemSubscriptionService>,
    wallet_store: Arc<dyn GemWalletStore>,
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
        platform: Arc<dyn GemDevicePlatform>,
        preferences: Arc<GemPreferencesService>,
    ) -> Self {
        Self {
            api,
            subscriptions,
            wallet_store,
            platform,
            preferences,
            sync_lock: Mutex::new(()),
        }
    }

    pub async fn set_push_enabled(&self, enabled: bool) -> Result<(), GemServiceError> {
        self.preferences.set_push_notifications_declined(!enabled)?;
        self.preferences.set_push_notifications_enabled(enabled)?;
        self.synchronize_if_needed().await
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
    pub async fn synchronize(&self) -> Result<Device, GemServiceError> {
        let _guard = self.sync_lock.lock().await;
        self.sync(self.current_device().await?).await
    }

    pub async fn is_registered(&self) -> Result<bool, GemServiceError> {
        Ok(self.preferences.is_device_registered())
    }
}

impl GemDeviceService {
    async fn current_device(&self) -> Result<Device, GemServiceError> {
        let info = self.platform.device_info().await?;
        let is_push_enabled = self.platform.is_push_enabled().await?;
        let token = match is_push_enabled {
            true => self.platform.push_token().await?,
            false => String::new(),
        };
        Ok(Device {
            id: self.platform.device_id().await?,
            platform: info.platform,
            platform_store: info.platform_store,
            os: info.os,
            model: info.model,
            token,
            locale: DeviceLocale::from_locale_identifier(&info.locale_identifier),
            version: info.version,
            currency: self.platform.get_currency().await?,
            is_push_enabled,
            is_price_alerts_enabled: Some(self.preferences.is_price_alerts_enabled()),
            subscriptions_version: 0,
        })
    }

    async fn needs_sync(&self, device: Device) -> Result<bool, GemServiceError> {
        if !self.preferences.is_device_registered() {
            return Ok(true);
        }
        let Some(pushed) = self.preferences.get_pushed_device() else {
            return Ok(true);
        };
        let local = Device {
            subscriptions_version: self.preferences.get_subscriptions_version(),
            ..device
        };
        if rules::device_changed(&pushed, &local) {
            return Ok(true);
        }
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets()?);
        Ok(self.preferences.get_pushed_subscriptions() != Some(signature))
    }

    async fn sync(&self, device: Device) -> Result<Device, GemServiceError> {
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets()?);
        let mut version = self.preferences.get_subscriptions_version();
        let remote = self.get_or_create(&device).await?;

        let signature_changed = self.preferences.get_pushed_subscriptions() != Some(signature.clone());
        if signature_changed || remote.subscriptions_version != version {
            self.subscriptions.sync().await?;
            if signature_changed {
                version += 1;
                self.preferences.set_subscriptions_version(version)?;
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
        self.preferences.set_pushed_device(&local)?;
        self.preferences.set_pushed_subscriptions(signature)?;
        Ok(synced)
    }
}

impl GemDeviceService {
    async fn get_or_create(&self, device: &Device) -> Result<Device, GemServiceError> {
        let registered = self.preferences.is_device_registered() || self.api.client.is_device_registered().await.map_err(GemApiError::from)?;
        if registered {
            match self.api.client.get_device().await.map_err(GemApiError::from)? {
                Some(remote) => {
                    self.preferences.set_device_registered(true)?;
                    return Ok(remote);
                }
                None => self.preferences.set_device_registered(false)?,
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
        self.preferences.set_device_registered(true)?;
        Ok(added)
    }
}

#[derive(Debug)]
pub(crate) struct DeviceSyncPreflight {
    pub(crate) service: Weak<GemDeviceService>,
}

#[async_trait::async_trait]
impl WalletRequestPreflight for DeviceSyncPreflight {
    async fn prepare(&self) -> Result<(), ClientError> {
        let Some(service) = self.service.upgrade() else {
            return Ok(());
        };
        service.synchronize_if_needed().await.map_err(|error| ClientError::Network(error.to_string()))
    }
}
