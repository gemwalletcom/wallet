use std::{error::Error, sync::Arc};

use cacher::SwapVaultAddressCacher;
use primitives::SwapProvider;

use super::proxy_client::SwapProxyClient;

pub struct NearIntentsProxyClient {
    client: SwapProxyClient,
}

impl NearIntentsProxyClient {
    pub fn new(url: String, deposit_addresses: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::NearIntents, "/quote/depositAddress"),
        }
    }

    pub async fn quote(&self, body: serde_json::Value) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/v0/quote", &body).await?)
    }
}
