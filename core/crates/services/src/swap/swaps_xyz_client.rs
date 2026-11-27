use std::error::Error;

use cacher::CacherClient;
use primitives::SwapProvider;
use swapper::swaps_xyz::ActionRequest;

use super::proxy_client::SwapProxyClient;

pub struct SwapsXyzProxyClient {
    client: SwapProxyClient,
}

impl SwapsXyzProxyClient {
    pub fn new(url: String, deposit_addresses: CacherClient) -> Self {
        Self {
            client: SwapProxyClient::new(url, deposit_addresses, SwapProvider::SwapsXyz),
        }
    }

    pub async fn action(&self, request: &ActionRequest) -> Result<gem_client::Response, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get("/getAction", request).await?)
    }
}
