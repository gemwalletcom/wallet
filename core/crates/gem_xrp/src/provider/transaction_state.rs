use async_trait::async_trait;
use chain_traits::ChainTransactionState;
use std::error::Error;

use gem_client::Client;
use primitives::{TransactionStateRequest, TransactionUpdate};

use crate::{provider::transaction_state_mapper::map_transaction_status, rpc::XrpClient};

#[async_trait]
impl<C: Client + Clone> ChainTransactionState for XrpClient<C> {
    async fn get_transaction_status(&self, request: TransactionStateRequest) -> Result<TransactionUpdate, Box<dyn Error + Sync + Send>> {
        let status = self.get_transaction_status(&request.id).await?;
        Ok(map_transaction_status(&status))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::create_xrp_test_client;
    use primitives::TransactionState;

    #[tokio::test]
    async fn test_get_transaction_status() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_xrp_test_client();
        let cases = [
            ("474F58E6C78F1DE8542036AB3C16E2B5A4089241DEE3E58142154DC3CA0E8271", TransactionState::Confirmed),
            ("5BC7CEDDA85478E819DF27214FF31785D3453A8E265B9D85360D2100B3902EDD", TransactionState::Failed),
        ];

        for (transaction_id, expected_state) in cases {
            let request = TransactionStateRequest::mock_with_id(transaction_id);
            let update = ChainTransactionState::get_transaction_status(&client, request).await?;
            assert_eq!(update.state, expected_state);
        }

        Ok(())
    }
}
