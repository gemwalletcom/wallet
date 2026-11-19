#[cfg(feature = "chain_integration_tests")]
use gem_client::ReqwestClient;
#[cfg(feature = "chain_integration_tests")]
use gem_evm::rpc::EthereumClient;
#[cfg(feature = "chain_integration_tests")]
use primitives::EVMChain;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

#[cfg(feature = "chain_integration_tests")]
use crate::staking::MonadStakingClient;

pub const TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

#[cfg(feature = "chain_integration_tests")]
pub fn create_staking_client() -> MonadStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    MonadStakingClient::new(EthereumClient::mock_with_url(EVMChain::Monad, &settings.chains.monad.url))
}
