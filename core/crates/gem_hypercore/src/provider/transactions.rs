use async_trait::async_trait;
use chain_traits::{ChainTransactions, TransactionsRequest};
use std::error::Error;

use gem_client::Client;
use primitives::Transaction;

use crate::{
    models::{order::UserFill, spot::SpotMeta},
    provider::transactions_mapper::map_user_fills,
    rpc::client::HyperCoreClient,
};

#[async_trait]
impl<C: Client> ChainTransactions for HyperCoreClient<C> {
    async fn get_transactions_by_address(&self, request: TransactionsRequest) -> Result<Vec<Transaction>, Box<dyn Error + Sync + Send>> {
        let start_time = request.from_timestamp.map(|ts| ts as i64 * 1000).unwrap_or(0);
        let fills = self.get_user_fills_by_time(&request.address, start_time).await?;
        let spot_meta = load_spot_meta_if_needed(self, &fills).await?;
        let transactions = map_user_fills(&request.address, fills, spot_meta.as_ref());

        match request.asset_id {
            Some(asset_id) => Ok(transactions.into_iter().filter(|tx| tx.asset_ids().contains(&asset_id)).collect()),
            None => Ok(transactions),
        }
    }

    async fn get_transaction_by_hash(&self, _hash: String) -> Result<Option<Transaction>, Box<dyn Error + Sync + Send>> {
        Ok(None)
    }
}

async fn load_spot_meta_if_needed<C: Client>(client: &HyperCoreClient<C>, fills: &[UserFill]) -> Result<Option<SpotMeta>, Box<dyn Error + Sync + Send>> {
    if fills.iter().any(|fill| fill.coin.starts_with('@')) {
        return Ok(Some(client.get_spot_meta().await?));
    }
    Ok(None)
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_TRANSACTION_ID, create_hypercore_test_client};

    #[tokio::test]
    async fn test_hypercore_get_transaction_by_hash() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_hypercore_test_client();
        assert!(client.get_transaction_by_hash(TEST_TRANSACTION_ID.to_string()).await?.is_none());

        Ok(())
    }
}
