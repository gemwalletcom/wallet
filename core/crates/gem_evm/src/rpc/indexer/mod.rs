use std::{error::Error, fmt, sync::Arc};

use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use primitives::{AssetBalance, EVMChain, try_in_order};

use super::{alchemy::AlchemyClient, ankr::AnkrClient, blockscout::BlockscoutClient, provider::AssetBalanceProvider};
use crate::provider::balances_mapper::map_assets_balances;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TransactionReference {
    pub(crate) hash: String,
    pub(crate) block_number: Option<u64>,
}

impl TransactionReference {
    pub(crate) fn new(hash: String, block_number: Option<u64>) -> Self {
        Self { hash, block_number }
    }
}

pub(crate) trait EVMIndexerClient {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>>;

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>>;
}

enum Provider<C: Client + Clone> {
    Alchemy(AlchemyClient<C>),
    Ankr(AnkrClient<C>),
    Blockscout(BlockscoutClient<C>),
}

enum ProviderKind {
    Alchemy,
    Ankr(&'static str),
    Blockscout,
}

#[derive(Debug)]
struct IndexerProviderError {
    provider: &'static str,
    source: Box<dyn Error + Send + Sync>,
}

impl IndexerProviderError {
    fn new(provider: &'static str, source: Box<dyn Error + Send + Sync>) -> Self {
        Self { provider, source }
    }
}

impl fmt::Display for IndexerProviderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.provider, self.source)
    }
}

impl Error for IndexerProviderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

impl<C: Client + Clone> Provider<C> {
    fn name(&self) -> &'static str {
        match self {
            Self::Alchemy(_) => "Alchemy",
            Self::Ankr(_) => "Ankr",
            Self::Blockscout(_) => "Blockscout",
        }
    }
}

impl<C: Client + Clone> EVMIndexerClient for Provider<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let result = match self {
            Self::Alchemy(client) => client.get_transactions_by_address(address, limit).await,
            Self::Ankr(client) => client.get_transactions_by_address(address, limit).await,
            Self::Blockscout(client) => client.get_transactions_by_address(address, limit).await,
        };
        result.map_err(|error| IndexerProviderError::new(self.name(), error).into())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let result = match self {
            Self::Alchemy(client) => client.get_token_balances(address).await,
            Self::Ankr(client) => client.get_token_balances(address).await,
            Self::Blockscout(client) => client.get_token_balances(address).await,
        };
        result.map_err(|error| IndexerProviderError::new(self.name(), error).into())
    }
}

pub struct EVMIndexer<C: Client + Clone> {
    providers: Vec<Provider<C>>,
    chain: EVMChain,
}

impl<C: Client + Clone> EVMIndexer<C> {
    pub fn for_chain(alchemy_client: C, ankr_client: C, blockscout_client: C, blockscout_key: String, chain: EVMChain) -> Option<Self> {
        let provider_kinds = match chain {
            EVMChain::Ethereum => vec![ProviderKind::Blockscout, ProviderKind::Ankr("eth")],
            EVMChain::Polygon => vec![ProviderKind::Blockscout, ProviderKind::Ankr("polygon")],
            EVMChain::AvalancheC => vec![ProviderKind::Ankr("avalanche")],
            EVMChain::SmartChain => vec![ProviderKind::Ankr("bsc")],
            EVMChain::Arbitrum => vec![ProviderKind::Blockscout, ProviderKind::Ankr("arbitrum")],
            EVMChain::Optimism => vec![ProviderKind::Blockscout, ProviderKind::Ankr("optimism")],
            EVMChain::Base => vec![ProviderKind::Blockscout, ProviderKind::Ankr("base")],
            EVMChain::Fantom => vec![ProviderKind::Ankr("fantom")],
            EVMChain::Gnosis => vec![ProviderKind::Blockscout, ProviderKind::Ankr("gnosis")],
            EVMChain::Linea => vec![ProviderKind::Ankr("linea")],
            EVMChain::XLayer => vec![ProviderKind::Ankr("xlayer")],
            EVMChain::ZkSync | EVMChain::Celo | EVMChain::World | EVMChain::Ink | EVMChain::Unichain | EVMChain::Robinhood => {
                vec![ProviderKind::Blockscout, ProviderKind::Alchemy]
            }
            EVMChain::Blast | EVMChain::Abstract | EVMChain::Berachain | EVMChain::Hyperliquid | EVMChain::Monad => vec![ProviderKind::Alchemy],
            EVMChain::OpBNB | EVMChain::Manta | EVMChain::Mantle | EVMChain::Sonic | EVMChain::SeiEvm | EVMChain::Plasma | EVMChain::Stable | EVMChain::Tempo => {
                return None;
            }
        };
        let provider = |kind| match kind {
            ProviderKind::Alchemy => Provider::Alchemy(AlchemyClient::new(JsonRpcClient::new(alchemy_client.clone()))),
            ProviderKind::Ankr(network) => Provider::Ankr(AnkrClient::new(JsonRpcClient::new(ankr_client.clone()), network)),
            ProviderKind::Blockscout => Provider::Blockscout(BlockscoutClient::new(blockscout_client.clone(), chain.chain_id(), blockscout_key.clone())),
        };
        Some(Self {
            providers: provider_kinds.into_iter().map(provider).collect(),
            chain,
        })
    }
}

pub struct EVMTransactionsByAddressProvider<C: Client + Clone> {
    indexer: Arc<EVMIndexer<C>>,
}

impl<C: Client + Clone> EVMTransactionsByAddressProvider<C> {
    pub fn new(indexer: Arc<EVMIndexer<C>>) -> Self {
        Self { indexer }
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for EVMTransactionsByAddressProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let operations = self
            .indexer
            .providers
            .iter()
            .map(|provider| provider.get_transactions_by_address(&request.address, request.limit))
            .collect::<Vec<_>>();
        let transactions = try_in_order(operations).await?.unwrap_or_default();
        Ok(TransactionsResult::TransactionRequests(
            transactions
                .into_iter()
                .map(|transaction| TransactionIdRequest::new(self.indexer.chain.to_chain(), transaction.hash, transaction.block_number))
                .collect(),
        ))
    }
}

pub struct EVMAssetBalanceProvider<C: Client + Clone> {
    indexer: Arc<EVMIndexer<C>>,
}

impl<C: Client + Clone> EVMAssetBalanceProvider<C> {
    pub fn new(indexer: Arc<EVMIndexer<C>>) -> Self {
        Self { indexer }
    }
}

#[async_trait]
impl<C: Client + Clone> AssetBalanceProvider for EVMAssetBalanceProvider<C> {
    async fn get_asset_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        let operations = self.indexer.providers.iter().map(|provider| provider.get_token_balances(&address)).collect::<Vec<_>>();
        let balances = try_in_order(operations).await?.unwrap_or_default();
        Ok(map_assets_balances(balances, self.indexer.chain.to_chain()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method;
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::Chain;

    #[tokio::test]
    async fn test_blockscout_falls_back_to_ankr() {
        let ankr_client = MockClient::new().with_post(|_, body| {
            let request: serde_json::Value = serde_json::from_slice(body).unwrap();
            let result: serde_json::Value = match request["method"].as_str().unwrap() {
                method::ANKR_GET_TRANSACTIONS_BY_ADDRESS => serde_json::from_str(include_str!("../../../testdata/ankr_get_transactions_by_address.json")).unwrap(),
                method::ANKR_GET_TOKEN_TRANSFERS => serde_json::from_str(include_str!("../../../testdata/ankr_get_token_transfers.json")).unwrap(),
                method => panic!("unexpected method: {method}"),
            };
            Ok(serde_json::to_vec(&serde_json::json!({ "jsonrpc": "2.0", "id": request["id"], "result": result })).unwrap())
        });
        let alchemy_client = ankr_client.clone();
        let blockscout_client = MockClient::new().with_get(|_| Err(ClientError::Http { status: 503, body: Vec::new() }));
        let indexer = EVMIndexer::for_chain(alchemy_client, ankr_client, blockscout_client, "key".to_string(), EVMChain::Ethereum).unwrap();
        let transactions_by_address = EVMTransactionsByAddressProvider::new(Arc::new(indexer));
        let result = transactions_by_address
            .get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 2))
            .await
            .unwrap();
        let transaction_ids = result.transaction_requests().unwrap();

        assert_eq!(
            transaction_ids,
            vec![
                TransactionIdRequest::new(Chain::Ethereum, "0xcee2abf4d8cc0ea0b9ecc9d21d81b7579f614a27a8740210856b199e5521f6f7".to_string(), None,),
                TransactionIdRequest::new(Chain::Ethereum, "0x1111111111111111111111111111111111111111111111111111111111111111".to_string(), None,)
            ]
        );
    }

    #[tokio::test]
    async fn test_provider_error_includes_provider_name() {
        let client = MockClient::new().with_post(|_, _| Err(ClientError::Http { status: 401, body: Vec::new() }));
        let provider = Provider::Alchemy(AlchemyClient::new(JsonRpcClient::new(client)));

        let error = provider.get_transactions_by_address("0x123", 1).await.unwrap_err();

        assert!(error.to_string().starts_with("Alchemy: "));
        assert!(error.source().is_some());
    }
}
