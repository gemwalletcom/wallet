use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{AssetBalance, TransactionFee, TransactionLoadInput};

use super::EthereumClient;
use crate::provider::preload_mapper::TransactionParams;

#[async_trait]
pub trait EvmFeeCalculator: Send + Sync {
    async fn calculate_fee(&self, input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>>;
}

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
    pub(crate) fee_calculator: Box<dyn EvmFeeCalculator>,
}

impl<C: Client + Clone> EthereumProvider<C> {
    pub fn new(client: EthereumClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>, asset_balance_provider: Box<dyn AssetBalanceProvider>) -> Self
    where
        C: 'static,
    {
        let fee_calculator = Box::new(client.clone());
        Self {
            client,
            transactions_by_address_provider,
            asset_balance_provider,
            fee_calculator,
        }
    }

    pub fn new_rpc_only(client: EthereumClient<C>) -> Self
    where
        C: 'static,
    {
        Self::new(client, Box::new(EmptyTransactionsProvider), Box::new(EmptyAssetBalanceProvider))
    }

    pub fn new_rpc_only_with_fee_calculator(client: EthereumClient<C>, fee_calculator: Box<dyn EvmFeeCalculator>) -> Self
    where
        C: 'static,
    {
        Self {
            fee_calculator,
            ..Self::new_rpc_only(client)
        }
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
    use chain_traits::ChainTraits;
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;
    use primitives::EVMChain;

    use super::*;

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
}
