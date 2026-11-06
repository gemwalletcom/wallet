use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainProvider, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use futures::{StreamExt, TryStreamExt, stream};
use gem_bitcoin::models::{Address, Transaction as BitcoinTransaction};
use gem_client::Client;
use primitives::Transaction;

use super::{
    ZcashProvider,
    transactions_mapper::{map_transaction, map_transactions},
};

const MAX_CONCURRENT_REQUESTS: usize = 5;

#[async_trait]
impl<C: Client> ChainBlockTransactions for ZcashProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let mut transactions = Vec::new();
        let mut page = 1;

        loop {
            let block = self.client.get_block(block, page).await?;
            let details = stream::iter(block.txs).map(|transaction| self.transaction_details(transaction)).buffer_unordered(MAX_CONCURRENT_REQUESTS).try_collect().await?;
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
        let has_transparent_input = transaction.vin.iter().any(|input| input.is_address);
        let is_shielded_spend = transaction.vin.is_empty() && transaction.vout.iter().any(|output| output.is_address);
        if !has_transparent_input && !is_shielded_spend {
            return Ok(transaction);
        }
        let txid = transaction.txid;
        self.client.get_transaction(&txid).await?.ok_or_else(|| format!("transaction {txid} not indexed").into())
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use num_bigint::BigUint;
    use primitives::{Chain, TransactionDirection};

    use super::*;
    use crate::testkit::{TEST_ZCASH_WALLET_ADDRESS, create_zcash_test_provider};

    const SHIELDED_SEND: &str = "1e056df81dc45aa0afe9ea32e751d9af91382d0fefe198ccfcee0f58e425a33a";
    const THORCHAIN_DEPOSIT: &str = "d45e295890061ef1107ba744a5b87e0b5c0d7c2293da8ac8015f11ce33ea4e8c";
    const NEAR_INTENTS_DEPOSIT: &str = "312557154abb972e9214db213b08e6085fdd2190069ec8c5d0c5e42b05dd0d40";

    fn summary(transaction: Transaction) -> (String, TransactionDirection, String, BigUint, BigUint) {
        let transaction = transaction.finalize(vec![TEST_ZCASH_WALLET_ADDRESS.to_string()]);
        (transaction.hash().to_string(), transaction.direction, transaction.to, transaction.value, transaction.fee)
    }

    #[tokio::test]
    async fn test_get_transactions_by_block() -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = create_zcash_test_provider();
        let expected = [
            (3_525_230, (SHIELDED_SEND.to_string(), TransactionDirection::Outgoing, String::new(), BigUint::from(200_000u64), BigUint::from(15_000u64))),
            (
                3_528_650,
                (
                    THORCHAIN_DEPOSIT.to_string(),
                    TransactionDirection::Outgoing,
                    "t1TpJ6FzYvXzHYFGDNhy1LeDQDaxMycTUNk".to_string(),
                    BigUint::from(500_000u64),
                    BigUint::from(25_000u64),
                ),
            ),
            (
                3_530_691,
                (
                    NEAR_INTENTS_DEPOSIT.to_string(),
                    TransactionDirection::Outgoing,
                    "t1extQZQJzFCSukTsKHkg4EABrGpiHPWZby".to_string(),
                    BigUint::from(200_000u64),
                    BigUint::from(10_000u64),
                ),
            ),
        ];
        for (block, expected) in expected {
            let transactions = provider.get_transactions_by_block(block).await?;
            let transaction = transactions.into_iter().find(|transaction| transaction.hash() == expected.0).unwrap();
            assert_eq!(summary(transaction), expected, "block {block}");
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let provider = create_zcash_test_provider();
        let transactions = match provider.get_transactions_by_address(TransactionsRequest::new(TEST_ZCASH_WALLET_ADDRESS.to_string(), 10)).await? {
            TransactionsResult::Transactions(transactions) => transactions,
            TransactionsResult::TransactionRequests(_) => vec![],
        };
        let summaries = transactions.into_iter().map(summary).filter(|summary| summary.0 == SHIELDED_SEND).collect::<Vec<_>>();
        assert_eq!(summaries, vec![(SHIELDED_SEND.to_string(), TransactionDirection::Outgoing, String::new(), BigUint::from(200_000u64), BigUint::from(15_000u64))]);

        let transaction = provider.get_transaction_by_hash(TransactionIdRequest::new(Chain::Zcash, SHIELDED_SEND.to_string(), None)).await?.unwrap();
        assert_eq!(summary(transaction).3, BigUint::from(200_000u64));
        Ok(())
    }
}
