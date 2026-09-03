use std::{collections::HashMap, error::Error};

use gem_client::{CONTENT_TYPE, ClientError, ClientExt, ContentType, ReqwestClient, json_response};
use primitives::FiatProviderName;
use reqwest::Method;

use super::model::{FlashnetEstimateResponse, FlashnetOnrampRequest, FlashnetOnrampResponse, FlashnetRoutesResponse};

pub struct FlashnetClient {
    client: ReqwestClient,
    pub(crate) api_key: String,
    pub(super) webhook_secret_key: String,
    pub(crate) affiliate_id: String,
}

impl FlashnetClient {
    pub const NAME: FiatProviderName = FiatProviderName::Flashnet;

    pub fn new(client: ReqwestClient, api_key: String, affiliate_id: String, webhook_secret_key: String) -> Self {
        Self {
            client,
            api_key,
            webhook_secret_key,
            affiliate_id,
        }
    }

    pub async fn get_routes(&self) -> Result<FlashnetRoutesResponse, Box<dyn Error + Send + Sync>> {
        let response = self.client.request(Method::GET, "/v1/orchestration/routes").send().await?;
        Ok(json_response(response).await?)
    }

    pub async fn create_onramp(&self, request: FlashnetOnrampRequest, idempotency_key: &str) -> Result<FlashnetOnrampResponse, ClientError> {
        self.client
            .post("/v1/orchestration/onramp", &request)
            .headers(HashMap::from([
                ("authorization".to_string(), format!("Bearer {}", self.api_key)),
                ("x-idempotency-key".to_string(), idempotency_key.to_string()),
                (CONTENT_TYPE.to_string(), ContentType::ApplicationJson.as_str().to_string()),
            ]))
            .await
    }

    pub async fn get_estimate(&self, destination_chain: &str, destination_asset: &str, amount: &str) -> Result<FlashnetEstimateResponse, Box<dyn Error + Send + Sync>> {
        let response = self
            .client
            .request(Method::GET, "/v1/orchestration/estimate")
            .bearer_auth(&self.api_key)
            .query(&[
                ("sourceChain", "spark"),
                ("sourceAsset", "USDB"),
                ("destinationChain", destination_chain),
                ("destinationAsset", destination_asset),
                ("amount", amount),
                ("affiliateId", self.affiliate_id.as_str()),
            ])
            .send()
            .await?;
        Ok(json_response(response).await?)
    }
}
