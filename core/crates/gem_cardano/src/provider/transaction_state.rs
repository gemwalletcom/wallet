use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::CardanoClient;

#[async_trait]
impl<C: Client> ChainTransactionState for CardanoClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let state = match self.get_transaction(&request.id).await? {
            Some(_) => TransactionState::Confirmed,
            None => TransactionState::Pending,
        };
        Ok(TransactionUpdate::new_state(state))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::testkit::MockClient;

    #[tokio::test]
    async fn test_get_transaction_status() {
        let included = MockClient::new().with_post(|_, _| Ok(br#"{"data":{"transactions":[{"hash":"tx_hash","includedAt":"2025-12-06T17:35:46Z","inputs":[],"outputs":[],"fee":"100"}]}}"#.to_vec()));
        let missing = MockClient::new().with_post(|_, _| Ok(br#"{"data":{"transactions":[]}}"#.to_vec()));

        assert_eq!(
            CardanoClient::new(included).get_transaction_status(TransactionStateRequest::mock_with_id("tx_hash")).await.unwrap(),
            TransactionUpdate::new_state(TransactionState::Confirmed)
        );
        assert_eq!(
            CardanoClient::new(missing).get_transaction_status(TransactionStateRequest::mock_with_id("tx_hash")).await.unwrap(),
            TransactionUpdate::new_state(TransactionState::Pending)
        );
    }
}
