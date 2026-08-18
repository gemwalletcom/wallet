pub const TEST_MONAD_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub use client::create_monad_staking_client;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
mod client {
    use gem_client::ReqwestClient;
    use gem_evm::rpc::EthereumClient;
    use primitives::EVMChain;
    use settings::testkit::get_test_settings;

    use crate::staking::MonadStakingClient;

    pub fn create_monad_staking_client() -> MonadStakingClient<ReqwestClient> {
        let settings = get_test_settings();
        MonadStakingClient::new(EthereumClient::mock_with_url(EVMChain::Monad, &settings.chains.monad.url))
    }
}
