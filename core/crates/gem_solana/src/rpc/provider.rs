use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainPerpetual, ChainProvider, ChainTransaction, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult};
use gem_client::Client;
use primitives::Chain;

use super::SolanaClient;

pub trait SolanaTransactionsIndexer: ChainTransactions + ChainTransaction {}

impl<T: ChainTransactions + ChainTransaction> SolanaTransactionsIndexer for T {}

pub struct SolanaProvider<C: Client + Clone> {
    client: SolanaClient<C>,
    pub(crate) indexer: Box<dyn SolanaTransactionsIndexer>,
}

impl<C: Client + Clone> SolanaProvider<C> {
    pub fn new(client: SolanaClient<C>, indexer: Box<dyn SolanaTransactionsIndexer>) -> Self {
        Self { client, indexer }
    }

    pub fn new_rpc_only(client: SolanaClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client + Clone> Deref for SolanaProvider<C> {
    type Target = SolanaClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for SolanaProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.indexer.get_transactions_by_address(request).await
    }
}

impl<C: Client + Clone> ChainProvider for SolanaProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }
}

impl<C: Client + Clone> ChainAccount for SolanaProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for SolanaProvider<C> {}
