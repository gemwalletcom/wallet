use std::error::Error;

use crate::models::{Block, TransactionLookup, Transactions};
use crate::rpc::target::AlgorandIndexerTarget;
use gem_client::{Client, ClientExt};

#[derive(Clone, Debug)]
pub struct AlgorandIndexer<C: Client> {
    pub client: C,
}

impl<C: Client> AlgorandIndexer<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub async fn get_account_transactions(&self, address: &str) -> Result<Transactions, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(AlgorandIndexerTarget::AccountTransactions { address: address.to_string() }).await?)
    }

    pub async fn get_block(&self, block_number: u64) -> Result<Block, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(AlgorandIndexerTarget::Block { number: block_number }).await?)
    }

    pub async fn get_transaction(&self, txid: &str) -> Result<TransactionLookup, Box<dyn Error + Send + Sync>> {
        Ok(self.client.get(AlgorandIndexerTarget::Transaction { id: txid.to_string() }).await?)
    }
}
