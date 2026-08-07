use std::{error::Error, ops::Deref};

use async_trait::async_trait;
use chain_traits::{
    ChainAccount, ChainAddressStatus, ChainPerpetual, ChainProvider, ChainSimulation, ChainTraits, ChainTransactions, EmptyTransactionsProvider, TransactionsRequest,
    TransactionsResult,
};
use gem_client::Client;
use primitives::Chain;

use super::PolkadotClient;

pub struct PolkadotProvider<C: Client> {
    client: PolkadotClient<C>,
    transactions_by_address_provider: Box<dyn ChainTransactions>,
}

impl<C: Client> PolkadotProvider<C> {
    pub fn new(client: PolkadotClient<C>, transactions_by_address_provider: Box<dyn ChainTransactions>) -> Self {
        Self {
            client,
            transactions_by_address_provider,
        }
    }

    pub fn new_rpc_only(client: PolkadotClient<C>) -> Self {
        Self::new(client, Box::new(EmptyTransactionsProvider))
    }
}

impl<C: Client> Deref for PolkadotProvider<C> {
    type Target = PolkadotClient<C>;

    fn deref(&self) -> &Self::Target {
        &self.client
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for PolkadotProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        self.transactions_by_address_provider.get_transactions_by_address(request).await
    }
}

impl<C: Client> ChainProvider for PolkadotProvider<C> {
    fn get_chain(&self) -> Chain {
        Chain::Polkadot
    }
}

impl<C: Client> ChainTraits for PolkadotProvider<C> {}
impl<C: Client> ChainAccount for PolkadotProvider<C> {}
impl<C: Client> ChainPerpetual for PolkadotProvider<C> {}
impl<C: Client> ChainAddressStatus for PolkadotProvider<C> {}
impl<C: Client> ChainSimulation for PolkadotProvider<C> {}
