use std::error::Error;

use futures::future::try_join_all;
use primitives::DefiPosition;
use storage::{Database, WalletsRepository};

use crate::config::DefiProviderConfig;
use crate::provider_client::DefiProviderClient;

pub struct DefiClient {
    database: Database,
    provider_client: DefiProviderClient,
}

impl DefiClient {
    pub fn from_config(database: Database, config: DefiProviderConfig) -> Self {
        Self {
            database,
            provider_client: DefiProviderClient::new(config),
        }
    }

    pub async fn get_positions_by_wallet_id(&self, device_id: i32, wallet_id: i32) -> Result<Vec<DefiPosition>, Box<dyn Error + Send + Sync>> {
        let subscriptions = self.database.wallets()?.get_subscriptions_by_wallet_id(device_id, wallet_id)?;
        let futures = subscriptions
            .iter()
            .map(|(subscription, address)| self.provider_client.get_positions(subscription.chain.0, &address.address));
        Ok(try_join_all(futures).await?.into_iter().flatten().collect())
    }
}
