use std::error::Error;

use defi::DefiProviderClient;
use futures::future::try_join_all;
use primitives::DefiPosition;
use storage::{Database, WalletsRepository};

pub struct DefiClient {
    database: Database,
    provider_client: DefiProviderClient,
}

impl DefiClient {
    pub fn new(database: Database, provider_client: DefiProviderClient) -> Self {
        Self { database, provider_client }
    }

    pub async fn get_positions_by_wallet_id(&self, device_id: i32, wallet_id: i32) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.run(move |client| client.get_subscriptions_by_wallet_id(device_id, wallet_id)).await?;
        let requests = subscriptions.iter().map(|subscription| self.provider_client.get_positions(subscription.chain, &subscription.address));
        Ok(try_join_all(requests).await?.into_iter().flatten().collect())
    }
}
