use std::error::Error;
use std::sync::Arc;

use defi::DefiProviderClient;
use futures::future::try_join_all;
use primitives::DefiPosition;

use super::repository::Repository;

pub struct DefiClient {
    repository: Arc<dyn Repository>,
    provider_client: DefiProviderClient,
}

impl DefiClient {
    pub(crate) fn new(repository: Arc<dyn Repository>, provider_client: DefiProviderClient) -> Self {
        Self { repository, provider_client }
    }

    pub async fn get_positions_by_wallet_id(&self, device_id: i32, wallet_id: i32) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.repository.get_wallet_subscriptions(device_id, wallet_id).await?;
        let requests = subscriptions.iter().map(|subscription| self.provider_client.get_positions(subscription.chain, &subscription.address));
        Ok(try_join_all(requests).await?.into_iter().flatten().collect())
    }
}
