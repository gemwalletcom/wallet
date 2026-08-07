use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainPerpetual, ChainProvider, ChainTraits, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::Chain;

use super::TronClient;
use crate::rpc::trongrid::model::TronGridAccount;

#[async_trait]
pub trait TronAccountProvider: Send + Sync {
    async fn get_accounts_by_address(&self, address: &str) -> Result<Vec<TronGridAccount>, Box<dyn Error + Send + Sync>>;
}

struct EmptyTronAccountProvider;

#[async_trait]
impl TronAccountProvider for EmptyTronAccountProvider {
    async fn get_accounts_by_address(&self, _address: &str) -> Result<Vec<TronGridAccount>, Box<dyn Error + Send + Sync>> {
        Ok(Vec::new())
    }
}

pub struct TronProvider<C: Client> {
    client: TronClient<C>,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
    account_provider: Box<dyn TronAccountProvider>,
}

impl<C: Client> TronProvider<C> {
    pub fn new(client: TronClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>, account_provider: Box<dyn TronAccountProvider>) -> Self {
        Self {
            client,
            transactions_by_address_provider,
            account_provider,
        }
    }

    pub fn new_rpc_only(client: TronClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider), Box::new(EmptyTronAccountProvider))
    }

    pub(crate) async fn get_indexer_accounts(&self, address: &str) -> Result<Vec<TronGridAccount>, Box<dyn Error + Send + Sync>> {
        self.account_provider.get_accounts_by_address(address).await
    }
}

impl<C: Client> Deref for TronProvider<C> {
    type Target = TronClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for TronProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transactions_by_address_provider.get_transactions_by_address(request).await
    }
}

impl<C: Client> ChainProvider for TronProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Tron
    }
}

impl<C: Client> ChainTraits for TronProvider<C> {}
impl<C: Client> ChainAccount for TronProvider<C> {}
impl<C: Client> ChainPerpetual for TronProvider<C> {}
