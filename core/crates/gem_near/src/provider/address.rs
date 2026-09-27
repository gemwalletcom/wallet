use std::error::Error;

use async_trait::async_trait;
use chain_traits::ChainAddressStatus;
use gem_client::Client;
use primitives::AddressStatus;

use crate::{address::address_to_public_key, provider::address_mapper, rpc::NearProvider};

#[async_trait]
impl<C: Client + Clone> ChainAddressStatus for NearProvider<C> {
    async fn get_address_status(&self, address: String) -> Result<Vec<AddressStatus>, Box<dyn Error + Sync + Send>> {
        let public_key = address_to_public_key(&address)?;
        let access_keys = self.get_account_access_keys(&address).await?;
        Ok(address_mapper::map_address_status(&public_key, &access_keys))
    }
}

#[cfg(test)]
mod tests {
    use gem_client::testkit::MockClient;
    use gem_jsonrpc::client::JsonRpcClient;

    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EXTERNALLY_CONTROLLED_ADDRESS};
    use crate::rpc::NearClient;

    #[tokio::test]
    async fn test_get_address_status() {
        for (address, response, expected) in [
            (TEST_ADDRESS, include_str!("../../testdata/access_key_list_full_access.json"), vec![]),
            (
                TEST_EXTERNALLY_CONTROLLED_ADDRESS,
                include_str!("../../testdata/access_key_list_externally_controlled.json"),
                vec![AddressStatus::ExternallyControlled],
            ),
            (TEST_ADDRESS, include_str!("../../testdata/access_key_list_function_call.json"), vec![AddressStatus::ExternallyControlled]),
        ] {
            let client = MockClient::new().with_post(move |_, _| Ok(response.as_bytes().to_vec()));
            let provider = NearProvider::new_rpc_only(NearClient::new(JsonRpcClient::new(client)));

            assert_eq!(provider.get_address_status(address.to_string()).await.unwrap(), expected);
        }
    }
}

#[cfg(all(test, feature = "chain_integration_tests"))]
mod chain_integration_tests {
    use super::*;
    use crate::provider::testkit::{TEST_ADDRESS, TEST_EXTERNALLY_CONTROLLED_ADDRESS, create_near_test_client};

    #[tokio::test]
    async fn test_get_address_status_regular() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_near_test_client();

        assert_eq!(client.get_address_status(TEST_ADDRESS.to_string()).await?, vec![]);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_address_status_without_derived_key() -> Result<(), Box<dyn Error + Send + Sync>> {
        let client = create_near_test_client();
        let status = client.get_address_status(TEST_EXTERNALLY_CONTROLLED_ADDRESS.to_string()).await?;

        assert_eq!(status, vec![AddressStatus::ExternallyControlled]);

        Ok(())
    }
}
