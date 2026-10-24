use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainProvider, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use futures::future::try_join_all;
use gem_bitcoin::models::{Address, Transaction as BitcoinTransaction};
use gem_client::Client;
use primitives::Transaction;

use super::{
    ZcashProvider,
    transactions_mapper::{map_transaction, map_transactions},
};

#[async_trait]
impl<C: Client> ChainBlockTransactions for ZcashProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let mut transactions = Vec::new();
        let mut page = 1;

        loop {
            let block = self.client.get_block(block, page).await?;
            let details = try_join_all(block.txs.into_iter().map(|transaction| self.transaction_details(transaction))).await?;
            transactions.extend(map_transactions(details));

            if block.total_pages == block.page {
                break;
            }

            page += 1;
        }

        Ok(transactions)
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for ZcashProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(transaction) = self.client.get_transaction(&request.hash).await? else {
            return Ok(None);
        };
        if transaction.block_height <= 0 || transaction.block_time <= 0 {
            return Ok(None);
        }

        Ok(map_transaction(&transaction))
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for ZcashProvider<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let address = Address::new(&address, self.get_chain()).full();
        let address_details = self.client.get_address_details(&address, limit).await?;
        Ok(TransactionsResult::Transactions(map_transactions(address_details.transactions.unwrap_or_default())))
    }
}

impl<C: Client> ZcashProvider<C> {
    async fn transaction_details(&self, transaction: BitcoinTransaction) -> Result<BitcoinTransaction, Box<dyn Error + Sync + Send>> {
        let has_transparent_address = transaction.vin.iter().any(|input| input.is_address) || transaction.vout.iter().any(|output| output.is_address);
        if !has_transparent_address {
            return Ok(transaction);
        }
        Ok(self.client.get_transaction(&transaction.txid).await?.unwrap_or(transaction))
    }
}
