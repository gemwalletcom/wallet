#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::{NearClient, NearIndexer, NearProvider};
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_client::ReqwestClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use gem_jsonrpc::client::JsonRpcClient;
#[cfg(all(test, feature = "chain_integration_tests"))]
use settings::testkit::get_test_settings;

#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_ADDRESS: &str = "75b4f90dc729b28ce1a3d44b2c96b3943136f1d7ced0b5df1fc23662439e3e3c";
#[cfg(all(test, feature = "chain_integration_tests"))]
pub const TEST_HISTORY_ADDRESS: &str = "root.near";

#[cfg(all(test, feature = "chain_integration_tests"))]
pub fn create_near_test_client() -> NearProvider<ReqwestClient> {
    let settings = get_test_settings();
    let url = settings.chains.near.url;
    let client = ReqwestClient::new(String::new(), gem_client::reqwest_client());
    NearProvider::new(
        NearClient::new(JsonRpcClient::new_reqwest(url)),
        Box::new(NearIndexer::new(
            settings.indexer.fastnear.transfers.remote_provider_config().configure_client(client.clone()),
            settings.indexer.fastnear.tx.remote_provider_config().configure_client(client),
        )),
    )
}
