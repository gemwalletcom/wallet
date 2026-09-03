use std::collections::HashMap;
use std::error::Error;

use gem_client::{Client, ClientExt};

use super::model::RatesResponse;
use super::target::TonApiTarget;

pub struct TonApiClient<C: Client> {
    client: C,
    api_key: String,
}

impl<C: Client> TonApiClient<C> {
    pub fn new(client: C, api_key: &str) -> Self {
        Self {
            client,
            api_key: api_key.to_string(),
        }
    }

    fn headers(&self) -> HashMap<String, String> {
        HashMap::from([("Authorization".to_string(), format!("Bearer {}", self.api_key))])
    }

    pub async fn get_rates(&self, tokens: &[String]) -> Result<RatesResponse, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(TonApiTarget::Rates { tokens: tokens.to_vec() }).headers(self.headers()).await?)
    }
}
