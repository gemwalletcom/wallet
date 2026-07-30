use gem_client::ReqwestClient;

use crate::JupiterProvider;

pub fn create_jupiter_test_provider() -> JupiterProvider {
    let settings = settings::testkit::get_test_settings();
    JupiterProvider::new(ReqwestClient::new_test_client(settings.prices.jupiter.url))
}
