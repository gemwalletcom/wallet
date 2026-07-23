#[cfg(all(test, feature = "chain_integration_tests"))]
use crate::rpc::{NearClient, NearIndexer};
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
pub fn create_near_test_client() -> NearClient<ReqwestClient> {
    let settings = get_test_settings();
    let url = settings.chains.near.url;
    let fastnear = settings.indexer.fastnear.remote_provider_config();
    NearClient::new(
        JsonRpcClient::new_reqwest(url),
        NearIndexer::new(fastnear.configure_client(ReqwestClient::new(String::new(), gem_client::reqwest_client())), fastnear.url),
    )
}
