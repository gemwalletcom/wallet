#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_evm::rpc::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::staking::MonadStakingClient;

pub const TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_staking_client() -> MonadStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    MonadStakingClient::new(EthereumClient::mock_with_url(EVMChain::Monad, &settings.chains.monad.url))
}
