use alloy_primitives::U256;
#[cfg(feature = "chain_integration_tests")]
use gem_evm::rpc::EthereumClient;
#[cfg(feature = "chain_integration_tests")]
use gem_jsonrpc::JsonRpcClient;
#[cfg(feature = "chain_integration_tests")]
use gem_reqwest::ReqwestClient;
use num_bigint::BigUint;
use num_traits::Zero;
#[cfg(feature = "chain_integration_tests")]
use primitives::EVMChain;
#[cfg(feature = "chain_integration_tests")]
use settings::testkit::get_test_settings;

use crate::contracts::IMonadStakingLens::{Delegation, DelegationState};
use crate::model::LensDelegation;
#[cfg(feature = "chain_integration_tests")]
use crate::staking::MonadStakingClient;

pub const TEST_ADDRESS: &str = "0x514BCb1F9AAbb904e6106Bd1052B66d2706dBbb7";

#[cfg(feature = "chain_integration_tests")]
pub fn create_staking_client() -> MonadStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    MonadStakingClient::new(EthereumClient::new(JsonRpcClient::new(ReqwestClient::new_test_client(settings.chains.monad.url)), EVMChain::Monad))
}

impl Delegation {
    pub fn mock() -> Self {
        Self {
            validatorId: 16,
            withdrawId: 0,
            state: DelegationState::Active,
            amount: U256::from(1_000_000_000_000_000_000u64),
            rewards: U256::ZERO,
            withdrawEpoch: 0,
            completionTimestamp: 0,
        }
    }
}

impl LensDelegation {
    pub fn mock_with(validator_id: u64, withdraw_id: u8, state: DelegationState) -> Self {
        Self {
            validator_id,
            withdraw_id,
            state,
            amount: BigUint::from(1_000_000_000_000_000_000u64),
            rewards: BigUint::zero(),
            completion_timestamp: 0,
        }
    }
}
