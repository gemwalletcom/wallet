use gem_client::ReqwestClient;

use crate::PythProvider;

pub fn create_pyth_test_provider() -> PythProvider {
    let settings = settings::testkit::get_test_settings();
    PythProvider::new(ReqwestClient::new_test_client(settings.prices.pyth.url))
}
