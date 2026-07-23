use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainTraits, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest, TransactionsResult,
};
use gem_client::Client;
use primitives::Chain;

use super::SolanaClient;

pub struct SolanaProvider<C: Client + Clone> {
    client: SolanaClient<C>,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
}

impl<C: Client + Clone> SolanaProvider<C> {
    pub fn new(client: SolanaClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>) -> Self {
        Self {
            client,
            transactions_by_address_provider,
        }
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
        self.transactions_by_address_provider.get_transactions_by_address(request).await
    }
}

impl<C: Client + Clone> ChainProvider for SolanaProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Solana
    }
}

impl<C: Client + Clone> ChainAccount for SolanaProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for SolanaProvider<C> {}
impl<C: Client + Clone> ChainAddressStatus for SolanaProvider<C> {}
impl<C: Client + Clone> ChainTraits for SolanaProvider<C> {}
