use std::error::Error;

use cacher::CacherClient;
use primitives::SwapProvider;

use super::proxy_client::SwapProxyClient;

pub struct RelayProxyClient {
    client: SwapProxyClient,
}

impl RelayProxyClient {
    pub fn new(url: String, deposit_addresses: CacherClient) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::Relay),
        }
    }

    pub async fn quote(&self, body: serde_json::Value) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        Ok(self.client.post("/quote/v2", &body).await?)
    }
}
