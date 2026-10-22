use std::{cmp::Reverse, collections::BTreeSet, error::Error, fmt, sync::Arc};

use alchemy::rpc::Client as AlchemyClient;
use ankr::Client as AnkrClient;
use async_trait::async_trait;
use blockscout::Client as BlockscoutClient;
use chain_traits::{ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use etherscan::client::EtherscanClient;
use gem_client::Client;
use gem_jsonrpc::client::JsonRpcClient;
use num_bigint::BigUint;
use primitives::{AssetBalance, EVMChain, try_in_order};

use super::etherscan::{self as etherscan_indexer, EtherscanClients};
use super::provider::AssetBalanceProvider;
use crate::provider::balances_mapper::map_assets_balances;

const MAX_PAGE_SIZE: usize = 50;
const TOKEN_BALANCES_UNSUPPORTED: &str = "token balances are not indexed";

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TransactionReference {
    pub(crate) hash: String,
    pub(crate) block_number: Option<u64>,
}

impl TransactionReference {
    pub(crate) fn new(hash: String, block_number: Option<u64>) -> Self {
        Self { hash, block_number }
    }

    pub(crate) fn unique_newest_first(references: impl IntoIterator<Item = (u64, String)>) -> Vec<Self> {
        references
            .into_iter()
            .map(|(block_number, hash)| (Reverse(block_number), hash))
            .collect::<BTreeSet<_>>()
            .into_iter()
            .map(|(Reverse(block_number), hash)| Self::new(hash, Some(block_number)))
            .collect()
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
    Etherscan(EtherscanClient<C>, EVMChain),
}

enum ProviderKind {
    Alchemy,
    Ankr(&'static str),
    Blockscout,
    Etherscan,
    Routescan,
    ZkSync,
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
            Self::Etherscan(..) => "Etherscan",
        }
    }
}

impl<C: Client + Clone> EVMIndexerClient for Provider<C> {
    async fn get_transactions_by_address(&self, address: &str, limit: usize) -> Result<Vec<TransactionReference>, Box<dyn Error + Send + Sync>> {
        let result = match self {
            Self::Alchemy(client) => client.get_transactions_by_address(address, limit).await,
            Self::Ankr(client) => client.get_transactions_by_address(address, limit).await,
            Self::Blockscout(client) => client.get_transactions_by_address(address, limit).await,
            Self::Etherscan(client, chain) => etherscan_indexer::get_transactions_by_address(client, *chain, address, limit).await,
        };
        result.map_err(|error| IndexerProviderError::new(self.name(), error).into())
    }

    async fn get_token_balances(&self, address: &str) -> Result<Vec<(String, BigUint)>, Box<dyn Error + Send + Sync>> {
        let result = match self {
            Self::Alchemy(client) => EVMIndexerClient::get_token_balances(client, address).await,
            Self::Ankr(client) => EVMIndexerClient::get_token_balances(client, address).await,
            Self::Blockscout(client) => EVMIndexerClient::get_token_balances(client, address).await,
            Self::Etherscan(..) => Err(TOKEN_BALANCES_UNSUPPORTED.into()),
        };
        result.map_err(|error| IndexerProviderError::new(self.name(), error).into())
    }
}

pub struct EVMIndexer<C: Client + Clone> {
    providers: Vec<Provider<C>>,
    chain: EVMChain,
}

impl<C: Client + Clone> EVMIndexer<C> {
    pub fn for_chain(alchemy_client: C, ankr_client: C, blockscout_client: C, blockscout_key: String, etherscan: EtherscanClients<C>, chain: EVMChain) -> Option<Self> {
        let provider_kinds = match chain {
            EVMChain::Ethereum => vec![ProviderKind::Etherscan, ProviderKind::Ankr("eth"), ProviderKind::Blockscout],
            EVMChain::Polygon => vec![ProviderKind::Etherscan, ProviderKind::Ankr("polygon"), ProviderKind::Blockscout],
            EVMChain::AvalancheC => vec![ProviderKind::Ankr("avalanche")],
            EVMChain::SmartChain => vec![ProviderKind::Ankr("bsc")],
            EVMChain::Arbitrum => vec![ProviderKind::Etherscan, ProviderKind::Ankr("arbitrum"), ProviderKind::Blockscout],
            EVMChain::Optimism => vec![ProviderKind::Ankr("optimism"), ProviderKind::Blockscout],
            EVMChain::Base => vec![ProviderKind::Ankr("base"), ProviderKind::Blockscout],
            EVMChain::Fantom => vec![ProviderKind::Ankr("fantom")],
            EVMChain::Gnosis => vec![ProviderKind::Ankr("gnosis"), ProviderKind::Blockscout],
            EVMChain::Linea => vec![ProviderKind::Ankr("linea")],
            EVMChain::XLayer => vec![ProviderKind::Ankr("xlayer")],
            EVMChain::Celo | EVMChain::World | EVMChain::Unichain => vec![ProviderKind::Etherscan, ProviderKind::Blockscout, ProviderKind::Alchemy],
            EVMChain::Ink | EVMChain::Robinhood | EVMChain::Arc => vec![ProviderKind::Blockscout, ProviderKind::Alchemy],
            EVMChain::Abstract | EVMChain::Berachain | EVMChain::Hyperliquid | EVMChain::Monad => vec![ProviderKind::Etherscan, ProviderKind::Alchemy],
            EVMChain::Blast => vec![ProviderKind::Routescan, ProviderKind::Alchemy],
            EVMChain::ZkSync => vec![ProviderKind::ZkSync, ProviderKind::Alchemy],
            EVMChain::OpBNB | EVMChain::Manta | EVMChain::Mantle | EVMChain::Sonic | EVMChain::SeiEvm | EVMChain::Plasma | EVMChain::Stable | EVMChain::Tempo => {
                return None;
            }
        };
        let provider = |kind| match kind {
            ProviderKind::Alchemy => Provider::Alchemy(AlchemyClient::new(JsonRpcClient::new(alchemy_client.clone()))),
            ProviderKind::Ankr(network) => Provider::Ankr(AnkrClient::new(JsonRpcClient::new(ankr_client.clone()), network)),
            ProviderKind::Blockscout => Provider::Blockscout(BlockscoutClient::new(blockscout_client.clone(), chain.chain_id(), blockscout_key.clone())),
            ProviderKind::Etherscan => Provider::Etherscan(etherscan.etherscan.clone(), chain),
            ProviderKind::Routescan => Provider::Etherscan(etherscan.routescan.clone(), chain),
            ProviderKind::ZkSync => Provider::Etherscan(etherscan.zksync.clone(), chain),
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
        let limit = request.limit.min(MAX_PAGE_SIZE);
        let operations = self.indexer.providers.iter().map(|provider| provider.get_transactions_by_address(&request.address, limit)).collect::<Vec<_>>();
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
    use ankr::testkit::{TOKEN_TRANSFERS, TRANSACTIONS};
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::{AssetId, Balance, Chain};
    use serde_json::{Value, from_slice, from_str, json, to_vec};

    use super::*;

    #[tokio::test]
    async fn test_etherscan_falls_back_to_ankr() {
        let ankr_client = MockClient::new().with_post(|_, body| {
            let request: Value = from_slice(body).unwrap();
            let result: Value = match request["method"].as_str().unwrap() {
                "ankr_getTransactionsByAddress" => from_str(TRANSACTIONS).unwrap(),
                "ankr_getTokenTransfers" => from_str(TOKEN_TRANSFERS).unwrap(),
                method => panic!("unexpected method: {method}"),
            };
            Ok(to_vec(&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result })).unwrap())
        });
        let alchemy_client = ankr_client.clone();
        let blockscout_client = MockClient::new().with_get(|_| Err(ClientError::Http { status: 503, body: Vec::new() }));
        let etherscan = EtherscanClients::new(blockscout_client.clone(), None, blockscout_client.clone(), blockscout_client.clone());
        let indexer = EVMIndexer::for_chain(alchemy_client, ankr_client, blockscout_client, "key".to_string(), etherscan, EVMChain::Ethereum).unwrap();
        let transactions_by_address = EVMTransactionsByAddressProvider::new(Arc::new(indexer));
        let result = transactions_by_address.get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 2)).await.unwrap();
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
    async fn test_etherscan_falls_back_to_alchemy() {
        let alchemy_client = MockClient::new().with_post(|_, body| {
            let request: Value = from_slice(body).unwrap();
            let result: Value = match request["method"].as_str().unwrap() {
                "alchemy_getAssetTransfers" if request["params"][0].get("fromAddress").is_some() => from_str(include_str!("../../../testdata/alchemy_get_asset_transfers_from.json")).unwrap(),
                "alchemy_getAssetTransfers" => from_str(include_str!("../../../testdata/alchemy_get_asset_transfers_to.json")).unwrap(),
                "alchemy_getTokenBalances" => from_str(include_str!("../../../testdata/alchemy_get_token_balances.json")).unwrap(),
                method => panic!("unexpected method: {method}"),
            };
            Ok(to_vec(&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result })).unwrap())
        });
        let routescan_client = MockClient::new().with_get(|_| Err(ClientError::Http { status: 503, body: Vec::new() }));
        let unused_client = MockClient::new();
        let etherscan = EtherscanClients::new(unused_client.clone(), None, routescan_client, unused_client.clone());
        let indexer = Arc::new(EVMIndexer::for_chain(alchemy_client, unused_client.clone(), unused_client, String::new(), etherscan, EVMChain::Blast).unwrap());

        let transactions = EVMTransactionsByAddressProvider::new(indexer.clone())
            .get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 2))
            .await
            .unwrap();
        let balances = EVMAssetBalanceProvider::new(indexer).get_asset_balances("0x123".to_string()).await.unwrap();

        assert_eq!(
            transactions.transaction_requests().unwrap(),
            vec![TransactionIdRequest::new(Chain::Blast, "0xin".to_string(), Some(3)), TransactionIdRequest::new(Chain::Blast, "0xout".to_string(), Some(2)),]
        );
        assert_eq!(
            balances,
            vec![AssetBalance::new_balance(
                AssetId::from_token(Chain::Blast, "0xdAC17F958D2ee523a2206206994597C13D831ec7"),
                Balance::coin_balance(BigUint::from(42u8))
            )]
        );
    }

    #[tokio::test]
    async fn test_etherscan_serves_history_before_blockscout() {
        let etherscan_client = MockClient::new().with_get(|path| {
            let response = match path {
                "/v2/api?chainid=42220&module=account&action=txlist&address=0x123&page=1&offset=3&sort=desc" => etherscan::testkit::TRANSACTIONS,
                "/v2/api?chainid=42220&module=account&action=tokentx&address=0x123&page=1&offset=3&sort=desc" => etherscan::testkit::TOKEN_TRANSFERS,
                _ => panic!("unexpected path: {path}"),
            };
            Ok(response.as_bytes().to_vec())
        });
        let blockscout_client = MockClient::new().with_get(|path| panic!("unexpected Blockscout path: {path}"));
        let unused_client = MockClient::new();
        let etherscan = EtherscanClients::new(etherscan_client, None, unused_client.clone(), unused_client.clone());
        let indexer = EVMIndexer::for_chain(unused_client.clone(), unused_client, blockscout_client, "key".to_string(), etherscan, EVMChain::Celo).unwrap();

        let transactions = EVMTransactionsByAddressProvider::new(Arc::new(indexer))
            .get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 3))
            .await
            .unwrap();

        assert_eq!(
            transactions.transaction_requests().unwrap(),
            vec![
                TransactionIdRequest::new(Chain::Celo, "0xtoken".to_string(), Some(11)),
                TransactionIdRequest::new(Chain::Celo, "0xnormal".to_string(), Some(10)),
                TransactionIdRequest::new(Chain::Celo, "0xshared".to_string(), Some(8)),
            ]
        );
    }

    #[tokio::test]
    async fn test_transaction_page_size_is_capped() {
        let client = MockClient::new().with_post(|_, body| {
            let request: Value = from_slice(body).unwrap();
            assert_eq!(request["params"]["pageSize"], MAX_PAGE_SIZE);
            let result: Value = match request["method"].as_str().unwrap() {
                "ankr_getTransactionsByAddress" => from_str(TRANSACTIONS).unwrap(),
                "ankr_getTokenTransfers" => from_str(TOKEN_TRANSFERS).unwrap(),
                method => panic!("unexpected method: {method}"),
            };
            Ok(to_vec(&json!({ "jsonrpc": "2.0", "id": request["id"], "result": result })).unwrap())
        });
        let etherscan = EtherscanClients::new(client.clone(), None, client.clone(), client.clone());
        let indexer = EVMIndexer::for_chain(client.clone(), client.clone(), client, String::new(), etherscan, EVMChain::SmartChain).unwrap();
        let provider = EVMTransactionsByAddressProvider::new(Arc::new(indexer));

        provider.get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), MAX_PAGE_SIZE + 1)).await.unwrap();
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
