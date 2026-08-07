use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits, ChainTransaction, ChainTransactions,
    EmptyTransactionsProvider, TransactionIdRequest, TransactionsRequest, TransactionsResult,
};
use gem_client::Client;
use primitives::{Chain, Transaction};

use super::AlgorandClient;

pub trait AlgorandTransactionProvider: ChainTransactions + ChainBlockTransactions + ChainTransaction {}

impl<T: ChainTransactions + ChainBlockTransactions + ChainTransaction> AlgorandTransactionProvider for T {}

pub struct AlgorandProvider<C: Client> {
    client: AlgorandClient<C>,
    transaction_provider: Box<dyn AlgorandTransactionProvider>,
}

impl<C: Client> AlgorandProvider<C> {
    pub fn new(client: AlgorandClient<C>, transaction_provider: Box<dyn AlgorandTransactionProvider>) -> Self {
        Self { client, transaction_provider }
    }

    pub fn new_rpc_only(client: AlgorandClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client> Deref for AlgorandProvider<C> {
    type Target = AlgorandClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for AlgorandProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transactions_by_address(request).await
    }
}

#[async_trait]
impl<C: Client> ChainBlockTransactions for AlgorandProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transactions_by_block(block).await
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for AlgorandProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        self.transaction_provider.get_transaction_by_hash(request).await
    }
}

impl<C: Client> ChainProvider for AlgorandProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Algorand
    }
}

impl<C: Client> ChainStaking for AlgorandProvider<C> {}
impl<C: Client> ChainAccount for AlgorandProvider<C> {}
impl<C: Client> ChainPerpetual for AlgorandProvider<C> {}
impl<C: Client> ChainAddressStatus for AlgorandProvider<C> {}
impl<C: Client> ChainSimulation for AlgorandProvider<C> {}
impl<C: Client> ChainTraits for AlgorandProvider<C> {}
