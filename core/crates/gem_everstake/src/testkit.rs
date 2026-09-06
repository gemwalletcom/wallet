use primitives::{AssetId, Chain, Delegation, DelegationBase, DelegationState, DelegationValidator};

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_client::ReqwestClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use gem_evm::rpc::EthereumClient;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use primitives::EVMChain;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use settings::testkit::get_test_settings;

#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::client::EverstakeClient;
use crate::constants::EVERSTAKE_POOL_ADDRESS;
#[cfg(all(feature = "rpc", feature = "reqwest"))]
use crate::staking::EverstakeStakingClient;

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
    }
}

#[cfg(all(feature = "rpc", feature = "reqwest"))]
pub fn create_everstake_staking_client() -> EverstakeStakingClient<ReqwestClient> {
    let settings = get_test_settings();
    EverstakeStakingClient::new(
        EthereumClient::mock_with_url(EVMChain::Ethereum, &settings.chains.ethereum.url),
        Some(EverstakeClient::new(ReqwestClient::new(settings.everstake.url, gem_client::reqwest_client()))),
    )
}
