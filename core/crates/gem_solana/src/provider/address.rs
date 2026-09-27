use async_trait::async_trait;
use chain_traits::ChainAddressStatus;
use gem_client::Client;
use primitives::AddressStatus;
use std::error::Error;

use crate::{provider::address_mapper, rpc::SolanaProvider};

#[async_trait]
impl<C: Client + Clone> ChainAddressStatus for SolanaProvider<C> {
    async fn get_address_status(&self, address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account_info_base64(&address).await?;
        Ok(address_mapper::map_address_status(account.value.as_ref()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_EMPTY_ADDRESS, create_solana_test_client};

    #[tokio::test]
    async fn test_get_address_status_missing_account() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();

        assert!(client.get_address_status(TEST_EMPTY_ADDRESS.to_string()).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_program_owned() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_solana_test_client();

        let status = client.get_address_status("HAgk14JpMQLgt6rVgv7cBQFJWFto5Dqxi472uT3DKpqk".to_string()).await?;

        assert_eq!(status, vec![AddressStatus::ExternallyControlled]);

        Ok(())
    }
}
