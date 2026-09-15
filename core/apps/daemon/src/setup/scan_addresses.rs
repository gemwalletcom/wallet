use gem_evm::{
    across::deployment::AcrossDeployment,
    uniswap::deployment::{
        get_uniswap_permit2_by_chain,
        v3::{
            get_aerodrome_router_deployment_by_chain, get_oku_deployment_by_chain, get_pancakeswap_router_deployment_by_chain, get_uniswap_router_deployment_by_chain,
            get_wagmi_router_deployment_by_chain,
        },
        v4::get_uniswap_deployment_by_chain,
    },
};
use gem_tracing::info_with_fields;
use primitives::{Chain, ScanAddress, SwapProvider};
use std::collections::HashSet;
use std::error::Error;
use storage::{Database, ScanAddressesRepository};
use swapper::{chainflip, mayan, near_intents, squid, thorchain::THORChainNetwork};

pub fn setup_scan_addresses(database: &Database) -> Result<(), Box<dyn Error + Send + Sync>> {
    let values = known_contracts();
    let count = values.len();
    let upserted = database.scan_addresses()?.upsert_scan_addresses(values)?;

    info_with_fields!("setup", step = "scan addresses", count = count, upserted = upserted);
    Ok(())
}

fn known_contracts() -> Vec<ScanAddress> {
    let mut contracts = Vec::new();

    for chain in Chain::all() {
        let uniswap_v3 = get_uniswap_router_deployment_by_chain(&chain);
        let uniswap_v4 = get_uniswap_deployment_by_chain(&chain);
        let pancakeswap = get_pancakeswap_router_deployment_by_chain(&chain);
        let oku = get_oku_deployment_by_chain(&chain);
        let wagmi = get_wagmi_router_deployment_by_chain(&chain);
        let aerodrome = get_aerodrome_router_deployment_by_chain(&chain);

        if let Some(address) = get_uniswap_permit2_by_chain(&chain) {
            contracts.push(ScanAddress::contract(chain, address, format!("{} Permit2", SwapProvider::UniswapV3.name())));
        }

        for (provider, address) in [
            (SwapProvider::UniswapV3, uniswap_v3.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::UniswapV4, uniswap_v4.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.universal_router)),
            (SwapProvider::Aerodrome, aerodrome.as_ref().map(|deployment| deployment.universal_router)),
        ] {
            if let Some(address) = address {
                contracts.push(ScanAddress::contract(chain, address, format!("{} Router", provider.name())));
            }
        }

        for (provider, address) in [
            (SwapProvider::PancakeswapV3, pancakeswap.as_ref().map(|deployment| deployment.permit2)),
            (SwapProvider::Oku, oku.as_ref().map(|deployment| deployment.permit2)),
            (SwapProvider::Wagmi, wagmi.as_ref().map(|deployment| deployment.permit2)),
        ] {
            if let Some(address) = address {
                contracts.push(ScanAddress::contract(chain, address, format!("{} Permit2", provider.name())));
            }
        }

        if let Some(deployment) = AcrossDeployment::deployment_by_chain(&chain) {
            for address in [deployment.spoke_pool, deployment.multicall_handler()] {
                contracts.push(ScanAddress::contract(chain, address, SwapProvider::Across.name()));
            }
        }
    }

    for network in [THORChainNetwork::Thorchain, THORChainNetwork::Mayachain] {
        contracts.extend(
            network
                .routers()
                .iter()
                .map(|(chain, router)| ScanAddress::contract(*chain, *router, network.provider().name())),
        );
    }
    contracts.extend(chainflip::VAULT_ADDRESSES.map(|(chain, vault)| ScanAddress::contract(chain, vault, SwapProvider::Chainflip.name())));
    contracts.extend(near_intents::TREASURY_ADDRESSES.map(|(chain, treasury)| ScanAddress::contract(chain, treasury, SwapProvider::NearIntents.name())));
    contracts.extend(
        mayan::MAYAN_DEPOSIT_CONTRACTS
            .into_iter()
            .chain(mayan::MAYAN_SEND_CONTRACTS)
            .map(|(chain, contract)| ScanAddress::contract(chain, contract, SwapProvider::Mayan.name())),
    );
    let (chain, multicall) = squid::SQUID_COSMOS_MULTICALL;
    contracts.push(ScanAddress::contract(chain, multicall, SwapProvider::Squid.name()));

    let mut seen = HashSet::new();
    contracts.retain(|contract| seen.insert((contract.chain, contract.address.clone())));
    contracts
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_known_contracts_are_named_by_provider_and_role() {
        let contracts = known_contracts();
        let names: HashMap<(Chain, &str), &str> = contracts
            .iter()
            .map(|contract| ((contract.chain, contract.address.as_str()), contract.name.as_deref().unwrap()))
            .collect();
        let uniswap = get_uniswap_router_deployment_by_chain(&Chain::SmartChain).unwrap();
        let pancakeswap = get_pancakeswap_router_deployment_by_chain(&Chain::SmartChain).unwrap();
        let across = AcrossDeployment::deployment_by_chain(&Chain::Ethereum).unwrap();

        assert_eq!(names[&(Chain::SmartChain, uniswap.permit2)], "Uniswap Permit2");
        assert_eq!(names[&(Chain::SmartChain, uniswap.universal_router)], "Uniswap Router");
        assert_eq!(names[&(Chain::SmartChain, pancakeswap.permit2)], "PancakeSwap Permit2");
        assert_eq!(names[&(Chain::SmartChain, pancakeswap.universal_router)], "PancakeSwap Router");
        assert_eq!(names[&(Chain::Ethereum, across.spoke_pool)], "Across");
        assert_eq!(names[&(Chain::Ethereum, across.multicall_handler())], "Across");
        for (chain, router) in THORChainNetwork::Thorchain.routers() {
            assert_eq!(names[&(*chain, *router)], "THORChain");
        }
        for (chain, router) in THORChainNetwork::Mayachain.routers() {
            assert_eq!(names[&(*chain, *router)], "Maya");
        }
        for (chain, vault) in chainflip::VAULT_ADDRESSES {
            assert_eq!(names[&(chain, vault)], "Chainflip");
        }
        for (chain, treasury) in near_intents::TREASURY_ADDRESSES {
            assert_eq!(names[&(chain, treasury)], "NEAR Intents");
        }
        for (chain, contract) in mayan::MAYAN_DEPOSIT_CONTRACTS.into_iter().chain(mayan::MAYAN_SEND_CONTRACTS) {
            assert_eq!(names[&(chain, contract)], "Mayan");
        }
        assert_eq!(names[&squid::SQUID_COSMOS_MULTICALL], "Squid");
    }
}
