use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};

use crate::constants::EVERSTAKE_POOL_ADDRESS;

pub const TEST_ETHEREUM_STAKING_ADDRESS: &str = gem_evm::testkit::TEST_ADDRESS;

pub fn mock_delegation(state: DelegationState) -> Delegation {
    Delegation {
        base: DelegationBase {
            asset_id: AssetId::from_chain(Chain::Ethereum),
            state,
            balance: 2_000_000_000_000_000_000u64.into(),
            shares: 0u32.into(),
            rewards: 0u32.into(),
            completion_date: None,
            delegation_id: "eth-delegation".to_string(),
            validator_id: EVERSTAKE_POOL_ADDRESS.to_string(),
        },
        validator: DelegationValidator::stake(Chain::Ethereum, EVERSTAKE_POOL_ADDRESS.to_string(), "Everstake".to_string(), true, 10.0, 4.2),
        price: None,
    }
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub use client::create_everstake_staking_client;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
mod client {
    use gem_client::ReqwestClient;
    use gem_evm::rpc::EthereumClient;
    use primitives::EVMChain;
    use settings::testkit::get_test_settings;

    use crate::staking::EverstakeStakingClient;

    pub fn create_everstake_staking_client() -> EverstakeStakingClient<ReqwestClient> {
        let settings = get_test_settings();
        EverstakeStakingClient::new(EthereumClient::mock_with_url(EVMChain::Ethereum, &settings.chains.ethereum.url))
    }
}
