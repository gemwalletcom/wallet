use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::AssetBalance;

use super::EthereumClient;
use super::chain_provider::EvmChainProvider;

#[async_trait]
pub trait AssetBalanceProvider: Send + Sync {
    async fn get_asset_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>>;
}

struct EmptyAssetBalanceProvider;

#[async_trait]
impl AssetBalanceProvider for EmptyAssetBalanceProvider {
    async fn get_asset_balances(&self, _address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        Ok(Vec::new())
    }
}

pub struct EthereumProvider<C: Client + Clone> {
    client: EthereumClient<C>,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
    asset_balance_provider: Box<dyn AssetBalanceProvider>,
    pub(crate) provider: Box<dyn EvmChainProvider>,
}

impl<C: Client + Clone> EthereumProvider<C> {
    pub fn new(client: EthereumClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>, asset_balance_provider: Box<dyn AssetBalanceProvider>) -> Self
    where
        C: 'static,
    {
        Self::new_with_provider(client.clone(), transactions_by_address_provider, asset_balance_provider, Box::new(client))
    }

    pub fn new_with_provider(
        client: EthereumClient<C>,
        transactions_by_address_provider: Box<dyn ChainTransactions>,
        asset_balance_provider: Box<dyn AssetBalanceProvider>,
        provider: Box<dyn EvmChainProvider>,
    ) -> Self
    where
        C: 'static,
    {
        Self {
            client,
            transactions_by_address_provider,
            asset_balance_provider,
            provider,
        }
    }

    pub fn new_rpc_only(client: EthereumClient<C>) -> Self
    where
        C: 'static,
    {
        Self::new(client, Box::new(EmptyTransactionsProvider), Box::new(EmptyAssetBalanceProvider))
    }

    pub fn new_rpc_only_with_provider(client: EthereumClient<C>, provider: Box<dyn EvmChainProvider>) -> Self
    where
        C: 'static,
    {
        Self::new_with_provider(client, Box::new(EmptyTransactionsProvider), Box::new(EmptyAssetBalanceProvider), provider)
    }

    pub(crate) async fn get_asset_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>> {
        self.asset_balance_provider.get_asset_balances(address).await
    }
}

impl<C: Client + Clone> Deref for EthereumProvider<C> {
    type Target = EthereumClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for EthereumProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transactions_by_address_provider.get_transactions_by_address(request).await
    }
}

#[cfg(test)]
mod tests {
    use chain_traits::{ChainBalances, ChainStaking, ChainTraits};
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;
    use primitives::EVMChain;

    use super::*;
    use crate::testkit::chain_provider_mock::MockChainProvider;

    #[tokio::test]
    async fn test_rpc_only_provider_returns_empty_transactions_and_asset_balances() {
        let client = EthereumClient::new(JsonRpcClient::new(MockClient::new()), EVMChain::OpBNB);
        let provider: Box<dyn ChainTraits> = Box::new(EthereumProvider::new_rpc_only(client));

        let transactions = match provider.get_transactions_by_address(TransactionsRequest::new("0x123".to_string(), 10)).await.unwrap() {
            TransactionsResult::Transactions(transactions) => transactions,
            TransactionsResult::TransactionRequests(_) => panic!("RPC-only provider must return an empty transaction list"),
        };
        assert!(transactions.is_empty());

        assert!(provider.get_balance_assets("0x123".to_string()).await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_provider_injection() {
        let make_client = || EthereumClient::new(JsonRpcClient::new(MockClient::new()), EVMChain::Ethereum);

        let ethereum = EthereumProvider::new_rpc_only(make_client());
        assert_eq!(ethereum.get_staking_apy().await.unwrap(), None);
        assert!(ethereum.get_staking_delegations("0x123".to_string()).await.unwrap().is_empty());
        assert_eq!(ethereum.get_balance_staking("0x123".to_string()).await.unwrap(), None);
        assert!(ethereum.provider.protocol_parser().is_none());

        let ethereum = EthereumProvider::new_rpc_only_with_provider(make_client(), Box::new(MockChainProvider));
        assert_eq!(ethereum.get_staking_apy().await.unwrap(), Some(42.0));
        assert!(ethereum.get_staking_delegations("0x123".to_string()).await.unwrap().is_empty());
        assert!(ethereum.get_balance_staking("0x123".to_string()).await.unwrap().is_some());
        assert!(ethereum.provider.protocol_parser().is_some());
    }
}
