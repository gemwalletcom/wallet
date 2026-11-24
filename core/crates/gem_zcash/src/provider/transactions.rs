use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainProvider, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use futures::{StreamExt, TryStreamExt, stream};
use gem_bitcoin::models::Address;
use gem_client::Client;
use primitives::{DateTime, Transaction};

use super::{
    ZcashProvider,
    transactions_mapper::{has_transparent_transfer, map_transaction, map_transactions},
};

const BLOCK_REQUEST_CONCURRENCY: usize = 5;

#[async_trait]
impl<C: Client> ChainBlockTransactions for ZcashProvider<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let block_transactions = self.client.get_block_transactions(block).await?;
        let transactions = stream::iter(block_transactions.into_iter().filter(has_transparent_transfer))
            .map(|transaction| async move {
                let txid = transaction.txid;
                self.client.get_transaction(&txid).await?.ok_or_else(|| Box::<dyn Error + Send + Sync>::from(format!("transaction {txid} not found")))
            })
            .buffered(BLOCK_REQUEST_CONCURRENCY)
            .try_collect()
            .await?;
        Ok(map_transactions(transactions))
    }
}

#[async_trait]
impl<C: Client> ChainTransaction for ZcashProvider<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(transaction) = self.client.get_transaction(&request.hash).await? else {
            return Ok(None);
        };
        if transaction.block_height <= 0 || transaction.block_time <= DateTime::UNIX_EPOCH {
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

#[cfg(test)]
mod tests {
    use gem_bitcoin::BitcoinClient;
    use gem_client::testkit::MockClient;
    use primitives::{BitcoinChain, TransactionDirection};

    use super::*;
    use crate::testkit::{TEST_RECEIVER_ADDRESS, TEST_SHIELDED_RECEIVE_ID, TEST_TRANSPARENT_RECIPIENT_ADDRESS, TEST_TRANSPARENT_SENDER_ADDRESS, TEST_TRANSPARENT_TRANSFER_ID, mock_shielded_receive, mock_transparent_transfer};

    #[tokio::test]
    async fn test_get_transactions_by_block() {
        let client = MockClient::new().with_get(|path| match path {
            "/api/v2/block/3500000?page=1" => Ok(include_bytes!("../../testdata/block_3500000.json").to_vec()),
            "/api/v2/block/3500001?page=1" => Ok(include_bytes!("../../testdata/block_3500001.json").to_vec()),
            path if path == format!("/api/v2/tx/{TEST_TRANSPARENT_TRANSFER_ID}") => Ok(include_bytes!("../../testdata/transaction_transparent.json").to_vec()),
            path if path == format!("/api/v2/tx/{TEST_SHIELDED_RECEIVE_ID}") => Ok(include_bytes!("../../testdata/transaction_shielded_receive.json").to_vec()),
            path => panic!("unexpected path: {path}"),
        });
        let provider = ZcashProvider::new(BitcoinClient::new(client, BitcoinChain::Zcash));
        let wallet = vec![TEST_TRANSPARENT_SENDER_ADDRESS.to_string(), TEST_RECEIVER_ADDRESS.to_string()];

        let transactions = provider.get_transactions_by_block(3_500_000).await.unwrap();

        assert_eq!(
            transactions.iter().map(|transaction| transaction.finalize(wallet.clone())).collect::<Vec<_>>(),
            vec![
                Transaction {
                    from: TEST_TRANSPARENT_SENDER_ADDRESS.to_string(),
                    to: TEST_TRANSPARENT_RECIPIENT_ADDRESS.to_string(),
                    direction: TransactionDirection::Outgoing,
                    ..mock_transparent_transfer()
                },
                Transaction {
                    to: TEST_RECEIVER_ADDRESS.to_string(),
                    direction: TransactionDirection::Incoming,
                    ..mock_shielded_receive()
                },
            ]
        );
        assert_eq!(provider.get_transactions_by_block(3_500_001).await.unwrap(), vec![]);
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use num_bigint::BigUint;
    use primitives::{Chain, TransactionDirection};

    use super::*;
    use crate::testkit::{TEST_ZCASH_WALLET_ADDRESS, create_zcash_test_provider};

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
        let near_intents_deposit = (
            NEAR_INTENTS_DEPOSIT.to_string(),
            TransactionDirection::Outgoing,
            "t1extQZQJzFCSukTsKHkg4EABrGpiHPWZby".to_string(),
            BigUint::from(200_000u64),
            BigUint::from(10_000u64),
        );
        let summaries = transactions.into_iter().map(summary).filter(|summary| summary.0 == NEAR_INTENTS_DEPOSIT).collect::<Vec<_>>();
        assert_eq!(summaries, vec![near_intents_deposit.clone()]);

        let transaction = provider.get_transaction_by_hash(TransactionIdRequest::new(Chain::Zcash, NEAR_INTENTS_DEPOSIT.to_string(), None)).await?.unwrap();
        assert_eq!(summary(transaction), near_intents_deposit);
        Ok(())
    }
}
