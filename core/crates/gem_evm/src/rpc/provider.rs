use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use num_bigint::BigInt;
use primitives::{AssetBalance, DelegationBase, DelegationValidator, StakeType, TransactionFee, TransactionLoadInput};

use super::EthereumClient;
use super::parsers::ProtocolParser;
use crate::provider::preload_mapper::TransactionParams;

#[async_trait]
pub trait AssetBalanceProvider: Send + Sync {
    async fn get_asset_balances(&self, address: String) -> Result<Vec<AssetBalance>, Box<dyn Error + Send + Sync>>;
}

#[async_trait]
pub trait EvmStakingClient: Send + Sync {
    async fn get_staking_apy(&self) -> Result<Option<f64>, Box<dyn Error + Sync + Send>>;
    async fn get_staking_validators(&self, apy: Option<f64>) -> Result<Vec<DelegationValidator>, Box<dyn Error + Sync + Send>>;
    async fn get_staking_delegations(&self, address: &str) -> Result<Vec<DelegationBase>, Box<dyn Error + Sync + Send>>;
    async fn get_staking_balance(&self, address: &str) -> Result<Option<AssetBalance>, Box<dyn Error + Sync + Send>>;
    fn encode_stake(&self, stake_type: &StakeType, value: &BigInt) -> Result<TransactionParams, Box<dyn Error + Sync + Send>>;
    fn node_check_method(&self) -> Option<&'static str> {
        None
    }
    async fn node_check_probe(&self, _address: &str) -> Result<(), Box<dyn Error + Sync + Send>> {
        Ok(())
    }
}

#[async_trait]
pub trait EvmFeeCalculator: Send + Sync {
    async fn calculate_fee(&self, input: &TransactionLoadInput, params: &TransactionParams, gas_limit: &BigInt) -> Result<TransactionFee, Box<dyn Error + Sync + Send>>;
}

#[derive(Default)]
pub struct EvmProviderExtensions {
    pub staking: Option<Box<dyn EvmStakingClient>>,
    pub fee_calculator: Option<Box<dyn EvmFeeCalculator>>,
    pub parsers: Vec<Box<dyn ProtocolParser>>,
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
    staking: Option<Box<dyn EvmStakingClient>>,
    fee_calculator: Option<Box<dyn EvmFeeCalculator>>,
    parsers: Vec<Box<dyn ProtocolParser>>,
}

impl<C: Client + Clone> EthereumProvider<C> {
    pub fn new(
        client: EthereumClient<C>,
        transactions_by_address_provider: Box<dyn ChainTransactions>,
        asset_balance_provider: Box<dyn AssetBalanceProvider>,
        extensions: EvmProviderExtensions,
    ) -> Self {
        Self {
            client,
            transactions_by_address_provider,
            asset_balance_provider,
            staking: extensions.staking,
            fee_calculator: extensions.fee_calculator,
            parsers: extensions.parsers,
        }
    }

    pub fn new_rpc_only(client: EthereumClient<C>) -> Self {
        Self::new_rpc_only_with_extensions(client, EvmProviderExtensions::default())
    }

    pub fn new_rpc_only_with_extensions(client: EthereumClient<C>, extensions: EvmProviderExtensions) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider), Box::new(EmptyAssetBalanceProvider), extensions)
    }

    pub(crate) fn client(&self) -> &EthereumClient<C> {
        &self.client
    }

    pub(crate) fn staking(&self) -> Option<&dyn EvmStakingClient> {
        self.staking.as_deref()
    }

    pub(crate) fn fee_calculator(&self) -> Option<&dyn EvmFeeCalculator> {
        self.fee_calculator.as_deref()
    }

    pub(crate) fn parsers(&self) -> &[Box<dyn ProtocolParser>] {
        &self.parsers
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
    use crate::testkit::staking_mock::MockStakingClient;

    #[tokio::test]
    async fn test_staking_client_injection() {
        let make_client = || EthereumClient::new(JsonRpcClient::new(MockClient::new()), EVMChain::Base);

        let provider = EthereumProvider::new_rpc_only(make_client());
        assert_eq!(provider.get_staking_apy().await.unwrap(), None);
        assert!(provider.get_staking_delegations("0x123".to_string()).await.unwrap().is_empty());
        assert_eq!(provider.get_balance_staking("0x123".to_string()).await.unwrap(), None);

        let provider = EthereumProvider::new_rpc_only_with_extensions(
            make_client(),
            EvmProviderExtensions {
                staking: Some(Box::new(MockStakingClient)),
                ..Default::default()
            },
        );
        assert_eq!(provider.get_staking_apy().await.unwrap(), Some(42.0));
        assert!(provider.get_staking_delegations("0x123".to_string()).await.unwrap().is_empty());
        assert!(provider.get_balance_staking("0x123".to_string()).await.unwrap().is_some());
    }

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
