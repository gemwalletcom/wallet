use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionState, TransactionStateRequest, TransactionUpdate};

use crate::rpc::client::CosmosClient;

use super::transaction_state_mapper::map_transaction_status;

#[async_trait]
impl<C: Client> ChainTransactionState for CosmosClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        Ok(match self.get_transaction(request.id).await? {
            Some(transaction) => map_transaction_status(transaction),
            None => TransactionUpdate::new_state(TransactionState::Pending),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gem_client::{ClientError, testkit::MockClient};
    use primitives::chain_cosmos::CosmosChain;

    #[tokio::test]
    async fn test_get_transaction_status() {
        let client = CosmosClient::new(
            CosmosChain::Celestia,
            MockClient::new().with_get(|_| {
                Err(ClientError::Http {
                    status: 404,
                    body: br#"{"code":5,"message":"tx not found"}"#.to_vec(),
                })
            }),
        );

        let update = client
            .get_transaction_status(TransactionStateRequest::mock_with_id("C6FBFF27349FCB0D339A54C715DA760CA6AB84AB1F4FB4FD7A15C35783939173"))
            .await
            .unwrap();

        assert_eq!(update, TransactionUpdate::new_state(TransactionState::Pending));
    }
}
