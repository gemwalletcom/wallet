use async_trait::async_trait;
use chain_traits::{ChainBlockTransactions, ChainTransaction, ChainTransactions, TransactionIdRequest, TransactionsRequest, TransactionsResult};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::models::rpc::{AccountLedger, Transaction as XrpTransaction};
use crate::provider::transactions_mapper::{batch_ledger, map_direct_transaction, map_transactions_by_address, map_transactions_by_block, split_batch_ledger};
use crate::rpc::XrpClient;

#[async_trait]
impl<C: Client + Clone> ChainBlockTransactions for XrpClient<C> {
    async fn get_transactions_by_block(&self, block: u64) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let ledger = self.get_block_transactions(block).await?;
        Ok(map_transactions_by_block(ledger))
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransaction for XrpClient<C> {
    async fn get_transaction_by_hash(&self, request: TransactionIdRequest) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        let Some(transaction) = self.get_transaction::<XrpTransaction>(&request.hash).await? else {
            return Ok(None);
        };
        let Some((account, ledger_index)) = batch_ledger(&transaction) else {
            return Ok(map_direct_transaction(self.get_chain(), transaction));
        };
        let ledger = self.get_ledger_account_transactions(account, ledger_index).await?;
        Ok(map_transactions_by_address(ledger).into_iter().find(|mapped| mapped.hash().eq_ignore_ascii_case(&request.hash)))
    }
}

#[async_trait]
impl<C: Client + Clone> ChainTransactions for XrpClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<TransactionsResult, Box<dyn Error + Sync + Send>> {
        let TransactionsRequest { address, limit, .. } = request;
        let page = self.get_account_transactions(address.clone(), limit).await?;
        let account_ledger = match split_batch_ledger(&page, limit) {
            Some(ledger_index) => AccountLedger {
                transactions: page
                    .transactions
                    .into_iter()
                    .filter(|transaction| u64::try_from(transaction.ledger_index).ok() != Some(ledger_index))
                    .chain(self.get_ledger_account_transactions(&address, ledger_index).await?.transactions)
                    .collect(),
            },
            None => page,
        };
        Ok(TransactionsResult::Transactions(map_transactions_by_address(account_ledger)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method;
    use gem_jsonrpc::testkit::mock_jsonrpc_client;
    use primitives::{Chain, TransactionType};
    use serde_json::{Value, json};

    const BATCH_SWAP_HASH: &str = "3B925B414BF7E91EC67A3E283F374DE0BC84D6E11DF5345115B44AF7DA5FEDEA";

    #[tokio::test]
    async fn test_get_transaction_by_hash_and_address() {
        let ledger: Value = serde_json::from_str(include_str!("../testdata/account_transactions_batch_swap.json")).unwrap();
        let page = json!({ "transactions": ledger["result"]["transactions"].as_array().unwrap()[..2] });
        let client = XrpClient::new(mock_jsonrpc_client(move |rpc_method, params| match (rpc_method, params[0].get("ledger_index")) {
            (method::TRANSACTION, _) => Ok(serde_json::from_str::<Value>(include_str!("../testdata/transaction_batch.json")).unwrap()["result"].clone()),
            (method::ACCOUNT_TRANSACTIONS, Some(Value::Number(_))) => Ok(ledger["result"].clone()),
            (method::ACCOUNT_TRANSACTIONS, _) => Ok(page.clone()),
            _ => panic!("unexpected method: {rpc_method}"),
        }));
        let swap = |transactions: Vec<Transaction>| {
            transactions
                .into_iter()
                .find(|transaction| transaction.transaction_type == TransactionType::Swap)
                .map(|transaction| transaction.hash().to_string())
        };

        let by_hash = client.get_transaction_by_hash(TransactionIdRequest::new(Chain::Xrp, BATCH_SWAP_HASH.to_lowercase(), None)).await.unwrap();
        let TransactionsResult::Transactions(by_address) = client.get_transactions_by_address(TransactionsRequest::new("rnCRC2AqXhFCnb9ULHtyCNrRYXgYuLBvZF".to_string(), 2)).await.unwrap() else {
            panic!("expected transactions");
        };

        assert_eq!(swap(by_hash.into_iter().collect()), Some(BATCH_SWAP_HASH.to_string()));
        assert_eq!(swap(by_address), Some(BATCH_SWAP_HASH.to_string()));
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_TRANSACTION_ID, create_xrp_test_client};

    #[tokio::test]
    async fn test_xrp_get_transactions_by_address() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let transactions = create_xrp_test_client().get_account_transactions(TEST_ADDRESS.to_string(), 1).await?.transactions;

        assert_eq!(transactions.len(), 1);
        assert!(!transactions[0].hash.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_xrp_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let transaction = client.get_transaction_by_hash(TransactionIdRequest::new(primitives::Chain::Xrp, TEST_TRANSACTION_ID.to_string(), None)).await?.unwrap();

        assert_eq!(transaction.hash(), TEST_TRANSACTION_ID);
        Ok(())
    }
}
