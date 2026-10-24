mod preload;
mod transactions;
mod transactions_mapper;
mod value_balance;

use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainAccount, ChainAddressStatus, ChainBalances, ChainPerpetual, ChainProvider, ChainSimulation, ChainStaking, ChainState, ChainToken, ChainTraits, ChainTransactionBroadcast, ChainTransactionState};
use gem_bitcoin::BitcoinClient;
use gem_client::Client;
use primitives::{AssetBalance, BroadcastOptions, Chain, NodeSyncStatus, TransactionStateRequest, TransactionUpdate};

pub struct ZcashProvider<C: Client> {
    client: BitcoinClient<C>,
}

impl<C: Client + 'static> ZcashProvider<C> {
    pub fn new(client: BitcoinClient<C>) -> Self {
        Self { client }
    }

    pub fn new_or_else(client: BitcoinClient<C>, fallback: impl FnOnce(BitcoinClient<C>) -> Box<dyn ChainTraits>) -> Box<dyn ChainTraits> {
        if client.get_chain() == Chain::Zcash { Box::new(Self::new(client)) } else { fallback(client) }
    }
}

impl<C: Client> ChainProvider for ZcashProvider<C> {
    fn get_chain(&self) -> Chain {
        self.client.get_chain()
    }
}

#[async_trait]
impl<C: Client> ChainBalances for ZcashProvider<C> {
    async fn get_balance_coin(&self, address: String) -> Result<AssetBalance, Box<dyn Error + Sync + Send>> {
        self.client.get_balance_coin(address).await
    }
}

#[async_trait]
impl<C: Client> ChainState for ZcashProvider<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        self.client.get_chain_id().await
    }

    async fn get_node_status(&self) -> Result<NodeSyncStatus, Box<dyn Error + Sync + Send>> {
        self.client.get_node_status().await
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        self.client.get_block_latest_number().await
    }
}

#[async_trait]
impl<C: Client> ChainTransactionState for ZcashProvider<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        self.client.get_transaction_status(request).await
    }
}

#[async_trait]
impl<C: Client> ChainTransactionBroadcast for ZcashProvider<C> {
    async fn transaction_broadcast(&self, data: String, options: BroadcastOptions) -> Result<String, Box<dyn Error + Sync + Send>> {
        self.client.transaction_broadcast(data, options).await
    }
}

#[async_trait]
impl<C: Client> ChainAccount for ZcashProvider<C> {}

impl<C: Client> ChainStaking for ZcashProvider<C> {}

impl<C: Client> ChainPerpetual for ZcashProvider<C> {}

impl<C: Client> ChainAddressStatus for ZcashProvider<C> {}

impl<C: Client> ChainToken for ZcashProvider<C> {}

impl<C: Client> ChainSimulation for ZcashProvider<C> {}

impl<C: Client> ChainTraits for ZcashProvider<C> {}
