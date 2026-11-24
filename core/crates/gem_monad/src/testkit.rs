#[cfg(feature = "chain_integration_tests")]
use gem_evm::rpc::EthereumClient;
#[cfg(feature = "chain_integration_tests")]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "chain_integration_tests")]
use gem_reqwest::ReqwestClient;
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
    MonadStakingClient::new(EthereumClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(settings.chains.monad.url)), EVMChain::Monad))
}
