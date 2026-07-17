use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::provider::transactions_mapper::map_transaction;
use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactions for CardanoClient<C> {
    async fn get_transactions_by_block(&self, block_number: u64) -> Result<Vec<Transaction>, Box<dyn Error + Send + Sync>> {
        let block = self.get_block(block_number).await?;
        let transactions = block
            .transactions
            .iter()
            .filter_map(|transaction| map_transaction(self.get_chain(), &block.forged_at, transaction))
            .collect::<Vec<Transaction>>();
        Ok(transactions)
    }

    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let transactions = self
            .get_address_transactions(&address, limit.unwrap_or(100))
            .await?
            .into_iter()
            .filter_map(|transaction| map_transaction(self.get_chain(), &transaction.included_at, &transaction.transaction))
            .collect();
        Ok(TransactionsResult::Transactions(transactions))
    }
}
