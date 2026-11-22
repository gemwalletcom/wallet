use gem_hypercore::rpc::client::HyperCoreClient;
use gem_reqwest::ReqwestClient;

use crate::HypercoreProvider;

pub fn create_hypercore_test_provider() -> HypercoreProvider {
    let settings = settings::testkit::get_test_settings();
    HypercoreProvider::new(HyperCoreClient::new(ReqwestClient::new_test_client(settings.prices.hypercore.url)))
}
