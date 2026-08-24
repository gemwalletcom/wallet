use std::{error::Error, slice};

use cacher::{CacheKey, CacherClient};
use gem_client::build_request_url;
use primitives::SwapProvider;
use swapper::swaps_xyz::{ActionRequest, ActionResponse};

pub struct SwapsXyzProxyClient {
    client: reqwest::Client,
    url: String,
    cacher: CacherClient,
}

impl SwapsXyzProxyClient {
    pub fn new(url: String, cacher: CacherClient) -> Self {
        Self {
            client: gem_client::reqwest_client(),
            url,
            cacher,
        }
    }

    pub async fn action(&self, request: &ActionRequest) -> Result<ActionResponse, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .get(build_request_url(&self.url, "/getAction"))
            .query(request)
            .send()
            .await?
            .error_for_status()?
            .json::<ActionResponse>()
            .await?;
        let _ = self
            .cacher
            .add_to_set_cached(CacheKey::SwapDepositAddresses(SwapProvider::SwapsXyz.as_ref()), slice::from_ref(&response.tx.to))
            .await;
        Ok(response)
    }
}
