use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use primitives::EVMChain;

use super::{alchemy::AlchemyClient, ankr::AnkrClient};

pub(crate) trait EVMIndexerClient {
    async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>;

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>>;
}

#[derive(Clone, Debug)]
enum Provider<C: Client + Clone> {
    Alchemy(AlchemyClient<C>),
    Ankr(AnkrClient<C>),
    Unsupported,
}

#[derive(Clone, Debug)]
pub struct EVMIndexer<C: Client + Clone> {
    provider: Provider<C>,
}

impl<C: Client + Clone> EVMIndexer<C> {
    pub fn new(ankr_client: JsonRpcClient<C>, alchemy_client: JsonRpcClient<C>, chain: EVMChain) -> Self {
        let provider = if let Some(client) = AnkrClient::new(ankr_client, chain) {
            Provider::Ankr(client)
        } else {
            Provider::Alchemy(AlchemyClient::new(alchemy_client))
        };
        Self { provider }
    }

    pub(crate) fn unsupported() -> Self {
        Self { provider: Provider::Unsupported }
    }
}

impl<C: Client + Clone> EVMIndexerClient for EVMIndexer<C> {
    async fn get_transaction_ids_by_address(&self, address: &str, limit: usize) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        match &self.provider {
            Provider::Alchemy(client) => client.get_transaction_ids_by_address(address, limit).await,
            Provider::Ankr(client) => client.get_transaction_ids_by_address(address, limit).await,
            Provider::Unsupported => Ok(Vec::new()),
        }
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        match &self.provider {
            Provider::Alchemy(client) => client.get_token_balances(address).await,
            Provider::Ankr(client) => client.get_token_balances(address).await,
            Provider::Unsupported => Ok(Vec::new()),
        }
    }
}
