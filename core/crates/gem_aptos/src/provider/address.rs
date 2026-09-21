use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainAddressStatus;
use gem_client::Client;
use primitives::AddressStatus;

use crate::provider::address_mapper;
use crate::rpc::client::AptosClient;

#[async_trait]
impl<C: Client> ChainAddressStatus for AptosClient<C> {
    async fn get_address_status(&self, address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        let account = self.get_account(&address).await?;
        Ok(address_mapper::map_address_status(&address, &account)?)
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, create_aptos_test_client};

    #[tokio::test]
    async fn test_get_address_status_regular() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();

        assert!(client.get_address_status(TEST_ADDRESS.to_string()).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_rotated_key() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_aptos_test_client();

        let status = client.get_address_status("0xffd7f0d7b24ba690657dd2aa7ccb754c4be938e4fb970c72eef966d664bd1a45".to_string()).await?;

        assert_eq!(status, vec![AddressStatus::ExternallyControlled]);

        Ok(())
    }
}
