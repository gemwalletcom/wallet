use std::error::Error;

use gem_client::{Client, ClientExt, ReqwestClient};
use primitives::EVMChain;
use serde::de::DeserializeOwned;

use crate::{
    model::{EtherscanResponse, GasOracle},
    target::EtherscanTarget,
};

#[derive(Clone)]
pub struct EtherscanClient<C: Client> {
    client: C,
    api_key: Option<String>,
}

impl EtherscanClient<ReqwestClient> {
    pub fn new_with_reqwest_client(client: reqwest::Client, api_key: Option<String>) -> Self {
        Self::new_with_client(ReqwestClient::new("https://api.etherscan.io".to_string(), client), api_key)
    }
}

impl<C: Client> EtherscanClient<C> {
    pub fn new_with_client(client: C, api_key: Option<String>) -> Self {
        Self { client, api_key }
    }

    pub async fn get_gas_oracle(&self, chain: EVMChain) -> Result<GasOracle, Box<dyn Error + Send + Sync>> {
        self.get(EtherscanTarget::GasOracle { chain }).await
    }

    async fn get<T: DeserializeOwned>(&self, target: EtherscanTarget) -> Result<T, Box<dyn Error + Send + Sync>> {
        let request = self.client.get::<EtherscanResponse>(target);
        let response = match &self.api_key {
            Some(api_key) => request.query(&[("apikey", api_key)]).await?,
            None => request.await?,
        };
        response.into_result()
    }
}
