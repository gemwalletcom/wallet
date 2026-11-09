use gem_bitcoin::BitcoinClient;
use gem_client::ReqwestClient;
use primitives::BitcoinChain;
use settings::testkit::get_test_settings;

use crate::ZcashProvider;

pub(crate) const TEST_ZCASH_WALLET_ADDRESS: &str = "t1dTLYuecTCt8irWJsyLkDaJm2EAvVgdxgM";

pub(crate) fn create_zcash_test_provider() -> ZcashProvider<ReqwestClient> {
    let settings = get_test_settings();
    ZcashProvider::new(BitcoinClient::new(ReqwestClient::new_test_client(settings.chains.zcash.url), BitcoinChain::Zcash))
}
