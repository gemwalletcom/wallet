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
        let provider = match chain {
            EVMChain::Ethereum => Provider::Ankr(AnkrClient::new(ankr_client, "eth")),
            EVMChain::Polygon => Provider::Ankr(AnkrClient::new(ankr_client, "polygon")),
            EVMChain::AvalancheC => Provider::Ankr(AnkrClient::new(ankr_client, "avalanche")),
            EVMChain::SmartChain => Provider::Ankr(AnkrClient::new(ankr_client, "bsc")),
            EVMChain::Arbitrum => Provider::Ankr(AnkrClient::new(ankr_client, "arbitrum")),
            EVMChain::Optimism => Provider::Ankr(AnkrClient::new(ankr_client, "optimism")),
            EVMChain::Base => Provider::Ankr(AnkrClient::new(ankr_client, "base")),
            EVMChain::Fantom => Provider::Ankr(AnkrClient::new(ankr_client, "fantom")),
            EVMChain::Gnosis => Provider::Ankr(AnkrClient::new(ankr_client, "gnosis")),
            EVMChain::Linea => Provider::Ankr(AnkrClient::new(ankr_client, "linea")),
            EVMChain::XLayer => Provider::Ankr(AnkrClient::new(ankr_client, "xlayer")),
            EVMChain::Blast
            | EVMChain::ZkSync
            | EVMChain::Celo
            | EVMChain::World
            | EVMChain::Abstract
            | EVMChain::Berachain
            | EVMChain::Ink
            | EVMChain::Unichain
            | EVMChain::Hyperliquid
            | EVMChain::Monad
            | EVMChain::Robinhood => Provider::Alchemy(AlchemyClient::new(alchemy_client)),
            EVMChain::OpBNB | EVMChain::Manta | EVMChain::Mantle | EVMChain::Sonic | EVMChain::SeiEvm | EVMChain::Plasma | EVMChain::Stable => Provider::Unsupported,
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

#[cfg(test)]
mod tests {
    use gem_jsonrpc::testkit::mock_jsonrpc_client;

    use super::*;

    #[tokio::test]
    async fn test_unsupported_indexer_does_not_call_provider() {
        let client = mock_jsonrpc_client(|method, _| panic!("unexpected RPC method: {method}"));
        let indexer = EVMIndexer::new(client.clone(), client, EVMChain::OpBNB);

        let transaction_ids = indexer.get_transaction_ids_by_address("0x123", 25).await.unwrap();

        assert_eq!(transaction_ids, Vec::<String>::new());
    }
}
