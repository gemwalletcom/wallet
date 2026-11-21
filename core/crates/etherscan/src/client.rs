use std::error::Error;

use gem_client::{Client, ClientExt};
use primitives::EVMChain;
use serde::de::DeserializeOwned;

use crate::{
    model::{EtherscanResponse, GasOracle, Transaction},
    target::{EtherscanApi, EtherscanTarget},
};

#[derive(Clone)]
pub struct EtherscanClient<C: Client> {
    client: C,
    api_key: Option<String>,
    api: EtherscanApi,
}

impl<C: Client> EtherscanClient<C> {
    pub fn new_with_client(client: C, api_key: Option<String>, api: EtherscanApi) -> Self {
        Self { client, api_key, api }
    }

    pub async fn get_gas_oracle(&self, chain: EVMChain) -> Result<GasOracle, Box<dyn Error + Send + Sync>> {
        self.get(EtherscanTarget::GasOracle { api: self.api, chain }).await
    }

    pub async fn get_transactions(&self, chain: EVMChain, address: &str, limit: usize) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get(EtherscanTarget::Transactions {
            api: self.api,
            chain,
            address: address.to_string(),
            limit,
        })
        .await
    }

    pub async fn get_token_transfers(&self, chain: EVMChain, address: &str, limit: usize) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        self.get(EtherscanTarget::TokenTransfers {
            api: self.api,
            chain,
            address: address.to_string(),
            limit,
        })
        .await
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
