use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};

use crate::stake_hub::BscUndelegation;

pub const TEST_SMARTCHAIN_STAKING_ADDRESS: &str = gem_evm::testkit::TEST_ADDRESS;

impl BscUndelegation {
    pub fn mock_with_unlock_time(unlock_time: &str) -> Self {
        BscUndelegation {
            delegator_address: TEST_SMARTCHAIN_STAKING_ADDRESS.to_string(),
            validator_address: "0x773760b0708a5Cc369c346993a0c225D8e4043B1".to_string(),
            amount: "1000000000000000000".to_string(),
            shares: "1000000000000000000".to_string(),
            unlock_time: unlock_time.to_string(),
        }
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
        price: None,
    }
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub use client::create_bsc_staking_client;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
mod client {
    use gem_client::ReqwestClient;
    use gem_evm::rpc::EthereumClient;
    use primitives::EVMChain;
    use settings::testkit::get_test_settings;

    use crate::staking::BscStakingClient;

    pub fn create_bsc_staking_client() -> BscStakingClient<ReqwestClient> {
        let settings = get_test_settings();
        BscStakingClient::new(EthereumClient::mock_with_url(EVMChain::SmartChain, &settings.chains.smartchain.url))
    }
}
