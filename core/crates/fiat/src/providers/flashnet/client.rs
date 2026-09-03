use std::{collections::HashMap, error::Error};

use gem_client::{ClientError, ClientExt, ReqwestClient};
use primitives::FiatProviderName;

use super::model::{EstimateQuery, FlashnetEstimateResponse, FlashnetOnrampRequest, FlashnetOnrampResponse, FlashnetRoutesResponse};
use super::target::FlashnetTarget;

const SOURCE_CHAIN: &str = "spark";
const SOURCE_ASSET: &str = "USDB";

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

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("authorization".to_string(), format!("Bearer {}", self.api_key))])
    }

    pub async fn get_routes(&self) -> Result<FlashnetRoutesResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(FlashnetTarget::Routes).await?)
    }

    pub async fn create_onramp(&self, request: FlashnetOnrampRequest, idempotency_key: &str) -> Result<FlashnetOnrampResponse, ClientError> {
        let mut headers = self.headers();
        headers.insert("x-idempotency-key".to_string(), idempotency_key.to_string());
        self.client.post(FlashnetTarget::Onramp, &request).headers(headers).await
    }

    pub async fn get_estimate(&self, destination_chain: &str, destination_asset: &str, amount: &str) -> Result<FlashnetEstimateResponse, Box<dyn Error + Send + Sync>> {
        let target = FlashnetTarget::Estimate {
            query: EstimateQuery {
                source_chain: SOURCE_CHAIN,
                source_asset: SOURCE_ASSET,
                destination_chain: destination_chain.to_string(),
                destination_asset: destination_asset.to_string(),
                amount: amount.to_string(),
                affiliate_id: self.affiliate_id.clone(),
            },
        };
        Ok(self.client.get(target).headers(self.headers()).await?)
    }
}
