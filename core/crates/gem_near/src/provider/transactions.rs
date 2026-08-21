use std::error::Error;

use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest, TransactionsResult};

use gem_client::Client;
use primitives::{Transaction, TransactionIdRequest};

use crate::rpc::NearIndexer;

#[async_trait]
impl<C: Client> ChainTransaction for NearIndexer<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        NearIndexer::get_transaction_by_hash(self, request).await
    }
}

#[async_trait]
impl<C: Client> ChainBlockTransactions for NearIndexer<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        NearIndexer::get_transactions_by_block(self, block).await
    }
}

#[async_trait]
impl<C: Client> ChainTransactions for NearIndexer<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest {
            address, limit, from_timestamp, ..
        } = request;
        let transactions = self.get_transactions_by_address(&address, limit, from_timestamp).await?;
        Ok(TransactionsResult::Transactions(transactions))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use std::error::Error;

    use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionsRequest};
    use primitives::{Chain, TransactionIdRequest, TransactionType, asset_constants::NEAR_USDT_ASSET_ID};

    use crate::provider::testkit::{TEST_HISTORY_ADDRESS, create_near_test_client};

    #[tokio::test]
    async fn test_near_get_transactions_by_address() -> Result<(), Box<dyn Error + Send + Sync>> {
        let result = create_near_test_client()
            .get_transactions_by_address(TransactionsRequest::new(TEST_HISTORY_ADDRESS.to_string(), 3))
            .await?;
        let transactions = result.transactions().ok_or("expected full NEAR transactions")?;

        assert_eq!(transactions.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_transaction_by_hash() -> Result<(), Box<dyn Error + Send + Sync>> {
        let hash = "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E";
        let transaction = create_near_test_client()
            .get_transaction_by_hash(TransactionIdRequest::new(Chain::Near, hash.to_string(), Some(211048907)))
            .await?
            .ok_or("expected NEAR transaction")?;

        assert_eq!(transaction.hash, hash);
        assert_eq!(transaction.block_number.as_deref(), Some("211048907"));
        assert_eq!(transaction.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(transaction.value, "99500026");
        Ok(())
    }

    #[tokio::test]
    async fn test_near_get_transactions_by_block() -> Result<(), Box<dyn Error + Send + Sync>> {
        let transactions = create_near_test_client().get_transactions_by_block(211048907).await?;

        let token_transfer = transactions
            .iter()
            .find(|transaction| transaction.hash == "DXUp65qSLjpbMrMVubtH1YY13fDHLA5av7q7skJ8kx5E")
            .ok_or("expected USDT transaction in NEAR block")?;
        assert_eq!(token_transfer.asset_id, NEAR_USDT_ASSET_ID.clone());
        assert_eq!(token_transfer.value, "99500026");

        let delegated_call = transactions
            .iter()
            .find(|transaction| transaction.hash == "G6RikjMH27TGcvntPjMZe1BcANDKCZgXREv6eSJZDTmC")
            .ok_or("expected delegated transaction in NEAR block")?;
        assert_eq!(delegated_call.transaction_type, TransactionType::SmartContractCall);
        assert_eq!(delegated_call.from, "0fc292ea67a5f50a3b2326ab5dcc395211f84bff66f0307da69e40c48d1b6ad3");
        assert_eq!(delegated_call.to, "v2.jars.sweat");
        Ok(())
    }
}
