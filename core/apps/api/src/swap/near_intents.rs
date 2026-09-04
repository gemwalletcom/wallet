use cacher::{CacheKey, CacherClient};
use gem_client::{ClientExt, ReqwestClient, Target};
use primitives::SwapProvider;

#[derive(Clone, Debug)]
enum NearIntentsProxyTarget {
    Quote,
}

impl Target for NearIntentsProxyTarget {
    fn path(&self) -> String {
        match self {
            Self::Quote => "/v0/quote".to_string(),
        }
    }
}

pub struct NearIntentsProxyClient {
    client: ReqwestClient,
    cacher: CacherClient,
}

impl NearIntentsProxyClient {
    pub fn new(url: String, cacher: CacherClient) -> Self {
        Self {
            client: ReqwestClient::new(url, gem_client::reqwest_client()),
            cacher,
        }
    }

    pub async fn quote(&self, body: serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error + Send + Sync>> {
        let response: serde_json::Value = self.client.post(NearIntentsProxyTarget::Quote, &body).await?;

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
