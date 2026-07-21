#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::{POLKADOT_ASSET_HUB_SUBSCAN_URL, PolkadotClient, PolkadotIndexer};
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;
#[cfg(all(test, feature = "chain_integration_tests"))]
use std::collections::HashMap;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "15oF4uVJwmo4TdGW7VfQxNLavjCXviqxT9S1MgbjMNHr6Sp5";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_polkadot_test_client() -> PolkadotClient<ReqwestClient> {
    let settings = get_test_settings();
    let client = gem_client::reqwest_client();
    let reqwest_client = ReqwestClient::new(settings.chains.polkadot.url, client.clone());
    PolkadotClient::new(
        reqwest_client,
        PolkadotIndexer::new(
            ReqwestClient::new(POLKADOT_ASSET_HUB_SUBSCAN_URL.to_string(), client).with_default_headers(HashMap::from([("x-api-key".to_string(), settings.subscan.key.secret)])),
        ),
    )
}
