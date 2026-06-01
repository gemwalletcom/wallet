#[cfg(feature = "rpc")]
use async_trait::async_trait;
#[cfg(feature = "rpc")]
use chain_traits::ChainTransactionState;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::client::SuiClient};

#[cfg(feature = "rpc")]
#[async_trait]
impl ChainTransactionState for SuiClient {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn std::error::Error + Sync + Send>> {
        let transaction = self.get_transaction(request.id).await?;
        Ok(map_transaction_status(transaction))
    }
}
