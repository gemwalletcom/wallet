use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use primitives::{TransactionStateRequest, TransactionUpdate};
use std::error::Error;

use gem_client::Client;

use crate::rpc::client::BitcoinClient;

use super::transaction_state_mapper::map_transaction_status;

#[async_trait]
impl<C: Client> ChainTransactionState for BitcoinClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let transaction = self.get_transaction(&request.id).await?;
        Ok(map_transaction_status(&transaction))
    }
}
