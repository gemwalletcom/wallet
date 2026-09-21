use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainAddressStatus;
use gem_client::Client;
use primitives::AddressStatus;

use crate::models::AccountResult;
use crate::provider::address_mapper;
use crate::rpc::client::StellarClient;

#[async_trait]
impl<C: Client> ChainAddressStatus for StellarClient<C> {
    async fn get_address_status(&self, address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        let account = match self.get_account(address.clone()).await? {
            AccountResult::Found(account) => Some(account),
            AccountResult::NotFound => None,
        };
        Ok(address_mapper::map_address_status(&address, account.as_ref()))
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EMPTY_ADDRESS, create_test_client};

    #[tokio::test]
    async fn test_get_address_status_regular() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        assert!(client.get_address_status(TEST_ADDRESS.to_string()).await?.is_empty());
        assert!(client.get_address_status(TEST_EMPTY_ADDRESS.to_string()).await?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_master_key_disabled() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let client = create_test_client();

        let status = client.get_address_status("GA5ZSEJYB37JRC5AVCIA5MOP4RHTM335X2KGX3IHOJAPP5RE34K4KZVN".to_string()).await?;

        assert_eq!(status, vec![AddressStatus::ExternallyControlled]);

        Ok(())
    }
}
