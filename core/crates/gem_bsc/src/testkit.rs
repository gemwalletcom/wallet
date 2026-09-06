use num_bigint::BigUint;
use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_evm::rpc::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

use crate::model::BscUndelegation;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::staking::BscStakingClient;

pub const TEST_SMARTCHAIN_STAKING_ADDRESS: &str = "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4";

pub fn mock_undelegation(unlock_time: Option<u64>) -> BscUndelegation {
    BscUndelegation {
        delegator_address: TEST_SMARTCHAIN_STAKING_ADDRESS.to_string(),
        validator_address: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
        amount: BigUint::from(1_000_000_000_000_000_000u64),
        shares: BigUint::from(1_000_000_000_000_000_000u64),
        unlock_time,
    }
}

pub fn mock_delegation(validator_id: &str, state: DelegationState, balance: u64, shares: u64) -> Delegation {
    Delegation {
        base: DelegationBase {
            asset_id: AssetId::from_chain(Chain::SmartChain),
            state,
            balance: balance.into(),
            shares: shares.into(),
            rewards: 0u32.into(),
            completion_date: None,
            delegation_id: "test".to_string(),
            validator_id: validator_id.to_string(),
        },
        validator: DelegationValidator::stake(Chain::SmartChain, validator_id.to_string(), "Test Validator".to_string(), true, 5.0, 10.0),
    }
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_staking_client() -> BscStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    BscStakingClient::new(EthereumClient::mock_with_url(EVMChain::SmartChain, &settings.chains.smartchain.url))
}
