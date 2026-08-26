use std::sync::Arc;

use primitives::{WalletSubscription, WalletSubscriptionChains};

use crate::api::{GemApiError, GemDeviceApiClient};

#[derive(Debug, uniffi::Object)]
pub struct GemSubscriptionService {
    api: Arc<GemDeviceApiClient>,
}

#[uniffi::export]
impl GemSubscriptionService {
    #[uniffi::constructor]
    pub fn new(api: Arc<GemDeviceApiClient>) -> Self {
        Self { api }
    }

    pub async fn get_subscriptions(&self) -> Result<Vec<WalletSubscriptionChains>, GemApiError> {
        Ok(self.api.client.get_subscriptions().await?)
    }

    pub async fn add_subscriptions(&self, subscriptions: Vec<WalletSubscription>) -> Result<(), GemApiError> {
        Ok(self.api.client.add_subscriptions(subscriptions).await?)
    }

    pub async fn delete_subscriptions(&self, subscriptions: Vec<WalletSubscriptionChains>) -> Result<(), GemApiError> {
        Ok(self.api.client.delete_subscriptions(subscriptions).await?)
    }
}
