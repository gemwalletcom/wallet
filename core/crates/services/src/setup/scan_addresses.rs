use gem_evm::{
    across::deployment::AcrossDeployment,
    uniswap::deployment::{
        get_uniswap_permit2_by_chain,
        v3::{get_aerodrome_router_deployment_by_chain, get_oku_deployment_by_chain, get_pancakeswap_router_deployment_by_chain, get_uniswap_router_deployment_by_chain, get_wagmi_router_deployment_by_chain},
        v4::get_uniswap_deployment_by_chain,
    },
};
use gem_tracing::info_with_fields;
use primitives::contract_constants::{UNISWAP_PERMIT2_CONTRACT, ZKSYNC_UNISWAP_PERMIT2_CONTRACT};
use primitives::{Chain, ScanAddress, SwapProvider};
use std::collections::HashMap;
use std::error::Error;
use swapper::{chainflip, mayan, near_intents, squid, thorchain::THORChainNetwork};

use super::repository::Repository;

pub(super) async fn setup_scan_addresses(repository: &dyn Repository) -> Result<(), Box<dyn Error + Send + Sync>> {
    let values = known_contracts().into_iter().map(|contract| ((contract.chain, contract.address.clone()), contract)).collect::<HashMap<_, _>>();
    let count = values.len();
    let inserted = repository.add_missing_scan_addresses(values).await?;

    info_with_fields!("setup", step = "scan addresses", count = count, inserted = inserted);
    Ok(())
}

fn known_contracts() -> Vec<ScanAddress> {
    Chain::all().into_iter().flat_map(swap_contracts).chain(provider_contracts()).collect()
}

fn swap_contracts(chain: Chain) -> Vec<ScanAddress> {
    let uniswap_v3 = get_uniswap_router_deployment_by_chain(&chain);
    let uniswap_v4 = get_uniswap_deployment_by_chain(&chain);
    let pancakeswap = get_pancakeswap_router_deployment_by_chain(&chain);
    let oku = get_oku_deployment_by_chain(&chain);
    let wagmi = get_wagmi_router_deployment_by_chain(&chain);
    let aerodrome = get_aerodrome_router_deployment_by_chain(&chain);

    let permit2 = [
        (SwapProvider::UniswapV3, get_uniswap_permit2_by_chain(&chain)),
        (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.permit2)),
        (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.permit2)),
        (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.permit2)),
    ]
    .into_iter()
    .filter_map(|(provider, address)| address.map(|address| ScanAddress::contract(chain, address, permit2_name(provider, address))));

    let routers = [
        (SwapProvider::UniswapV3, uniswap_v3.as_ref().map(|deployment| deployment.universal_router)),
        (SwapProvider::UniswapV4, uniswap_v4.as_ref().map(|deployment| deployment.universal_router)),
        (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.universal_router)),
        (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.universal_router)),
        (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.universal_router)),
        (SwapProvider::Aerodrome, aerodrome.as_ref().map(|deployment| deployment.universal_router)),
    ]
    .into_iter()
    .filter_map(|(provider, address)| address.map(|address| ScanAddress::contract(chain, address, format!("{} Router", provider.name()))));

    let across = AcrossDeployment::deployment_by_chain(&chain)
        .into_iter()
        .flat_map(|deployment| [deployment.spoke_pool, deployment.multicall_handler()])
        .map(|address| ScanAddress::contract(chain, address, SwapProvider::Across.name()));

    permit2.chain(routers).chain(across).collect()
}

fn provider_contracts() -> Vec<ScanAddress> {
    let routers = [THORChainNetwork::Thorchain, THORChainNetwork::Mayachain]
        .into_iter()
        .flat_map(|network| network.routers().iter().map(move |(chain, router)| ScanAddress::contract(*chain, *router, network.provider().name())));
    let vaults = chainflip::VAULT_ADDRESSES
        .into_iter()
        .map(|(chain, vault)| (SwapProvider::Chainflip, chain, vault))
        .chain(near_intents::TREASURY_ADDRESSES.into_iter().map(|(chain, treasury)| (SwapProvider::NearIntents, chain, treasury)))
        .chain(mayan::MAYAN_DEPOSIT_CONTRACTS.into_iter().chain(mayan::MAYAN_SEND_CONTRACTS).map(|(chain, contract)| (SwapProvider::Mayan, chain, contract)))
        .chain([(SwapProvider::Squid, squid::SQUID_COSMOS_MULTICALL.0, squid::SQUID_COSMOS_MULTICALL.1)])
        .map(|(provider, chain, address)| ScanAddress::contract(chain, address, provider.name()));

    routers.chain(vaults).collect()
}

fn permit2_name(provider: SwapProvider, address: &str) -> String {
    match address {
        UNISWAP_PERMIT2_CONTRACT | ZKSYNC_UNISWAP_PERMIT2_CONTRACT => "Permit2".to_string(),
        _ => format!("{} Permit2", provider.name()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_a_contract_listed_more_than_once_keeps_one_name() {
        let contracts = known_contracts();
        let conflicting = contracts
            .iter()
            .filter(|contract| contracts.iter().any(|other| other.chain == contract.chain && other.address == contract.address && other.name != contract.name))
            .collect::<Vec<_>>();

        assert!(conflicting.is_empty(), "{conflicting:?}");
    }

    #[test]
    fn test_the_shared_permit2_is_named_permit2_whichever_provider_lists_it() {
        let contracts = known_contracts();
        let name = |chain: Chain, address: &str| contracts.iter().find(|contract| contract.chain == chain && contract.address == address).and_then(|contract| contract.name.clone());
        let pancakeswap = get_pancakeswap_router_deployment_by_chain(&Chain::SmartChain).unwrap();

        assert_eq!(name(Chain::Ethereum, UNISWAP_PERMIT2_CONTRACT).as_deref(), Some("Permit2"));
        assert_eq!(name(Chain::Gnosis, UNISWAP_PERMIT2_CONTRACT).as_deref(), Some("Permit2"), "only Oku lists it on Gnosis");
        assert_eq!(name(Chain::ZkSync, ZKSYNC_UNISWAP_PERMIT2_CONTRACT).as_deref(), Some("Permit2"));
        assert_eq!(name(Chain::SmartChain, pancakeswap.permit2).as_deref(), Some("PancakeSwap Permit2"));
        assert_eq!(name(Chain::SmartChain, pancakeswap.universal_router).as_deref(), Some("PancakeSwap Router"));
    }
}
