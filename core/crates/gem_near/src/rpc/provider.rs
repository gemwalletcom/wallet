use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainBlockTransactions, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainTraits, ChainTransaction, ChainTransactions,
    EmptyTransactionsProvider, TransactionsRequest, TransactionsResult,
};
use gem_client::Client;
use primitives::Chain;

use super::NearClient;

pub struct NearProvider<C: Client + Clone> {
    client: NearClient<C>,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
}

impl<C: Client + Clone> NearProvider<C> {
    pub fn new(client: NearClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>) -> Self {
        Self {
            client,
            transactions_by_address_provider,
        }
    }

    pub fn new_rpc_only(client: NearClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client + Clone> Deref for NearProvider<C> {
    type Target = NearClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for NearProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transactions_by_address_provider.get_transactions_by_address(request).await
    }
}

impl<C: Client + Clone> ChainProvider for NearProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Near
    }
}

impl<C: Client + Clone> ChainStaking for NearProvider<C> {}
impl<C: Client + Clone> ChainPerpetual for NearProvider<C> {}
impl<C: Client + Clone> ChainAddressStatus for NearProvider<C> {}
impl<C: Client + Clone> ChainAccount for NearProvider<C> {}
impl<C: Client + Clone> ChainSimulation for NearProvider<C> {}
impl<C: Client + Clone> ChainTransaction for NearProvider<C> {}
impl<C: Client + Clone> ChainBlockTransactions for NearProvider<C> {}
impl<C: Client + Clone> ChainTraits for NearProvider<C> {}
