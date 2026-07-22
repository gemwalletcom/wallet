use std::error::Error;

use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use primitives::{EVMChain, try_in_order};

use super::{alchemy::AlchemyClient, ankr::AnkrClient, blockscout::BlockscoutClient};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IndexedTransaction {
    pub(crate) hash: String,
    pub(crate) block_number: Option<u64>,
}

impl IndexedTransaction {
    pub(crate) fn new(hash: String, block_number: Option<u64>) -> Self {
        Self { hash, block_number }
    }
}

pub(crate) trait EVMIndexerClient {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<IndexedTransaction>, Box<dyn Error + Send + Sync>>;

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>>;
}

#[derive(Clone, Debug)]
enum Provider<C: Client + Clone> {
    Alchemy(AlchemyClient<C>),
    Ankr(AnkrClient<C>),
    Blockscout(BlockscoutClient<C>),
}

impl<C: Client + Clone> EVMIndexerClient for Provider<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<IndexedTransaction>, Box<dyn Error + Send + Sync>> {
        match self {
            Self::Alchemy(client) => client.get_transactions_by_address(address, limit).await,
            Self::Ankr(client) => client.get_transactions_by_address(address, limit).await,
            Self::Blockscout(client) => client.get_transactions_by_address(address, limit).await,
        }
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        match self {
            Self::Alchemy(client) => client.get_token_balances(address).await,
            Self::Ankr(client) => client.get_token_balances(address).await,
            Self::Blockscout(client) => client.get_token_balances(address).await,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EVMIndexer<C: Client + Clone> {
    providers: Vec<Provider<C>>,
}

impl<C: Client + Clone> EVMIndexer<C> {
    pub fn new(alchemy_client: JsonRpcClient<C>, ankr_client: JsonRpcClient<C>, blockscout_client: C, blockscout_key: String, chain: EVMChain) -> Self {
        let alchemy = || Provider::Alchemy(AlchemyClient::new(alchemy_client));
        let ankr = |network| Provider::Ankr(AnkrClient::new(ankr_client, network));
        let blockscout = |client| Provider::Blockscout(BlockscoutClient::new(client, chain.chain_id(), blockscout_key));
        let providers = match chain {
            EVMChain::Ethereum => vec![blockscout(blockscout_client), ankr("eth")],
            EVMChain::Polygon => vec![blockscout(blockscout_client), ankr("polygon")],
            EVMChain::AvalancheC => vec![ankr("avalanche")],
            EVMChain::SmartChain => vec![ankr("bsc")],
            EVMChain::Arbitrum => vec![blockscout(blockscout_client), ankr("arbitrum")],
            EVMChain::Optimism => vec![blockscout(blockscout_client), ankr("optimism")],
            EVMChain::Base => vec![blockscout(blockscout_client), ankr("base")],
            EVMChain::Fantom => vec![ankr("fantom")],
            EVMChain::Gnosis => vec![blockscout(blockscout_client), ankr("gnosis")],
            EVMChain::Linea => vec![ankr("linea")],
            EVMChain::XLayer => vec![ankr("xlayer")],
            EVMChain::ZkSync | EVMChain::Celo | EVMChain::World | EVMChain::Ink | EVMChain::Unichain | EVMChain::Robinhood => {
                vec![blockscout(blockscout_client), alchemy()]
            }
            EVMChain::Blast | EVMChain::Abstract | EVMChain::Berachain | EVMChain::Hyperliquid | EVMChain::Monad => vec![alchemy()],
            EVMChain::OpBNB | EVMChain::Manta | EVMChain::Mantle | EVMChain::Sonic | EVMChain::SeiEvm | EVMChain::Plasma | EVMChain::Stable => Vec::new(),
        };
        Self { providers }
    }

    pub(crate) fn unsupported() -> Self {
        Self { providers: Vec::new() }
    }
}

impl<C: Client + Clone> EVMIndexerClient for EVMIndexer<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<IndexedTransaction>, Box<dyn Error + Send + Sync>> {
        let operations = self
            .providers
            .iter()
            .map(|provider| provider.get_transactions_by_address(address, limit))
            .collect::<Vec<_>>();
        match try_in_order(operations).await? {
            Some(transaction_ids) => Ok(transaction_ids),
            None => Ok(Vec::new()),
        }
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let operations = self.providers.iter().map(|provider| provider.get_token_balances(address)).collect::<Vec<_>>();
        match try_in_order(operations).await? {
            Some(balances) => Ok(balances),
            None => Ok(Vec::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::method;
    use gem_client::{ClientError, testkit::MockClient};
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::testkit::json::load_json;

    use super::*;

    #[tokio::test]
    async fn test_unsupported_indexer_does_not_call_provider() {
        let client = mock_jsonrpc_client(|method, _| panic!("unexpected RPC method: {method}"));
        let blockscout_client = MockClient::new().with_get(|path| panic!("unexpected Blockscout path: {path}"));
        let indexer = EVMIndexer::new(client.clone(), client, blockscout_client, "key".to_string(), EVMChain::OpBNB);

        let transaction_ids = indexer.get_transactions_by_address("0x123", 25).await.unwrap();

        assert_eq!(transaction_ids, Vec::<IndexedTransaction>::new());
    }

    #[tokio::test]
    async fn test_blockscout_falls_back_to_ankr() {
        let ankr_client = mock_jsonrpc_client(|request_method, _| match request_method {
            method::ANKR_GET_TRANSACTIONS_BY_ADDRESS => Ok(load_json(include_str!("../../../testdata/ankr_get_transactions_by_address.json"))),
            method::ANKR_GET_TOKEN_TRANSFERS => Ok(load_json(include_str!("../../../testdata/ankr_get_token_transfers.json"))),
            _ => panic!("unexpected method: {request_method}"),
        });
        let alchemy_client = ankr_client.clone();
        let blockscout_client = MockClient::new().with_get(|_| Err(ClientError::Http { status: 503, body: Vec::new() }));
        let indexer = EVMIndexer::new(alchemy_client, ankr_client, blockscout_client, "key".to_string(), EVMChain::Ethereum);

        let transaction_ids = indexer.get_transactions_by_address("0x123", 2).await.unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                IndexedTransaction::new("0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string(), None),
                IndexedTransaction::new("0x1111111111111111111111111111111111111111111111111111111111111111".to_string(), None)
            ]
        );
    }
}
