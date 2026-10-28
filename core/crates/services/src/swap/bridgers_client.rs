use std::{error::Error, sync::Arc};

use cacher::SwapVaultAddressCacher;
use primitives::SwapProvider;

use super::proxy_client::SwapProxyClient;

pub struct BridgersProxyClient {
    client: SwapProxyClient,
}

impl BridgersProxyClient {
    pub fn new(url: String, deposit_addresses: Arc<dyn SwapVaultAddressCacher>) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::Bridgers),
        }
    }

    pub async fn swap(&self, body: serde_json::Value) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/api/sswap/swap", &body).await?)
    }
}
