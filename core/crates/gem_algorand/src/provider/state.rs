use async_trait::async_trait;
use chain_traits::ChainState;
use std::error::Error;

use gem_client::Client;

use crate::rpc::AlgorandProvider;

#[async_trait]
impl<C: Client> ChainState for AlgorandProvider<C> {
    async fn get_chain_id(&self) -> Result<Option<String>, Box<dyn Error + Sync + Send>> {
        Ok(Some(self.get_transactions_params().await?.genesis_id))
    }

    async fn get_block_latest_number(&self) -> Result<u64, Box<dyn Error + Sync + Send>> {
        Ok(self.get_transactions_params().await?.last_round)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use chain_traits::ChainState;
    use primitives::Chain;

    use crate::provider::testkit::*;

    #[tokio::test]
    async fn test_algorand_get_chain_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let chain_id = client.get_chain_id().await?;

        assert_eq!(chain_id.as_deref(), Some(Chain::Algorand.network_id()));
        Ok(())
    }

    #[tokio::test]
    async fn test_algorand_get_block_latest_number() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_algorand_test_client();
        let latest_block = client.get_block_latest_number().await?;
        println!("Latest block: {}", latest_block);
        assert!(latest_block > 0);
        Ok(())
    }
}
