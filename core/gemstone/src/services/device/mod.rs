pub mod rules;
pub mod store;

use crate::services::error::GemServiceError;
use std::sync::Arc;

use primitives::Device;

pub use store::GemDeviceStore;

use crate::api::{GemApiError, GemDeviceApiClient};
use crate::services::subscription::GemSubscriptionService;
use crate::services::wallet::GemWalletStore;

#[derive(uniffi::Object)]
pub struct GemDeviceService {
    api: Arc<GemDeviceApiClient>,
    subscriptions: Arc<GemSubscriptionService>,
    wallet_store: Arc<dyn GemWalletStore>,
    store: Arc<dyn GemDeviceStore>,
}

#[uniffi::export]
impl GemDeviceService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>, subscriptions: Arc<GemSubscriptionService>, wallet_store: Arc<dyn GemWalletStore>, store: Arc<dyn GemDeviceStore>) -> Self {
        Self {
            api,
            subscriptions,
            wallet_store,
            store,
        }
    }

    pub async fn needs_sync(&self, device: Device) -> Result<bool, GemServiceError> {
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
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets().await?);
        Ok(self.store.get_pushed_subscriptions().await? != Some(signature))
    }

    pub async fn sync(&self, device: Device) -> Result<Device, GemServiceError> {
        let signature = rules::subscriptions_signature(&self.wallet_store.get_wallets().await?);
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
