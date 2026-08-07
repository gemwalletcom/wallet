use std::collections::HashMap;
use std::error::Error;

use gem_client::{ClientExt, ReqwestClient};

use super::model::RatesResponse;

pub struct TonApiClient {
    client: ReqwestClient,
}

impl TonApiClient {
    pub fn new(client: ReqwestClient, api_key: &str) -> Self {
        Self {
            client: client.with_default_headers(HashMap::from([("Authorization".to_string(), format!("Bearer {api_key}"))])),
        }
    }

    pub async fn get_rates(&self, tokens: &[String]) -> Result<RatesResponse, Box<dyn Error + Send + Sync>> {
        let query = vec![("tokens".to_string(), tokens.join(",")), ("currencies".to_string(), "usd".to_string())];
        Ok(self.client.get_with_query("/v2/rates", &query).await?)
    }
}
