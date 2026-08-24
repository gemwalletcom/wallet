use cacher::{CacheKey, CacherClient};
use gem_client::build_request_url;
use primitives::SwapProvider;

pub struct NearIntentsProxyClient {
    client: reqwest::Client,
    url: String,
    cacher: CacherClient,
}

impl NearIntentsProxyClient {
    pub fn new(url: String, cacher: CacherClient) -> Self {
        let client = gem_client::reqwest_client();
        Self { client, url, cacher }
    }

    pub async fn quote(&self, body: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let url = build_request_url(&self.url, "/v0/quote");
        let response = self.client.post(&url).json(&body).send().await?.json::<serde_json::Value>().await?;

        if let Some(address) = response.pointer("/quote/depositAddress").and_then(|v| v.as_str())
            && !address.is_empty()
        {
            let _ = self
                .cacher
                .add_to_set_cached(CacheKey::SwapDepositAddresses(SwapProvider::NearIntents.as_ref()), &[address.to_string()])
                .await;
        }

        Ok(response)
    }
}
