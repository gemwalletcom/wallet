use gem_evm::{
    across::deployment::AcrossDeployment,
    uniswap::deployment::{
        aero, get_uniswap_permit2_by_chain,
        v3::{get_aerodrome_router_deployment_by_chain, get_oku_deployment_by_chain, get_pancakeswap_router_deployment_by_chain, get_uniswap_router_deployment_by_chain, get_wagmi_router_deployment_by_chain},
        v4::get_uniswap_deployment_by_chain,
    },
};
use gem_tracing::info_with_fields;
use primitives::contract_constants::{TRON_SUNSWAP_PERMIT2_CONTRACT, TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT, UNISWAP_PERMIT2_CONTRACT, ZKSYNC_UNISWAP_PERMIT2_CONTRACT};
use primitives::{AddressType, Chain, ChainType, ScanAddress, SwapProvider};
use std::collections::HashMap;
use std::error::Error;
use swapper::{bridgers, chainflip, mayan, near_intents, okx, relay, squid, thorchain::THORChainNetwork};

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
        (SwapProvider::Aero, aero::get_factory_by_chain(&chain).map(|_| aero::METAROUTER)),
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
        .flat_map(|network| network.routers().iter().map(move |router| ScanAddress::contract(router.chain, router.contract, network.provider().name())));
    let contracts = chainflip::VAULT_ADDRESSES
        .iter()
        .map(|vault| (SwapProvider::Chainflip, vault.chain, vault.contract))
        .chain(
            mayan::MAYAN_DEPOSIT_CONTRACTS
                .iter()
                .chain(mayan::MAYAN_SEND_CONTRACTS)
                .map(|contract| (SwapProvider::Mayan, contract.chain, contract.contract)),
        )
        .chain([(SwapProvider::Squid, squid::SQUID_COSMOS_MULTICALL.chain, squid::SQUID_COSMOS_MULTICALL.contract)])
        .map(|(provider, chain, address)| ScanAddress::contract(chain, address, provider.name()));
    let treasuries = near_intents::TREASURY_ADDRESSES.into_iter().map(|(chain, treasury)| ScanAddress {
        address_type: Some(AddressType::Address),
        ..ScanAddress::contract(chain, treasury, SwapProvider::NearIntents.name())
    });
    let okx = okx::DEPLOYMENTS.iter().flat_map(|deployment| {
        [(deployment.contract, "OKX DEX Router"), (deployment.spender, "OKX DEX Approval")]
            .into_iter()
            .map(move |(address, name)| ScanAddress::contract(deployment.chain, address, if deployment.chain == Chain::Solana { SwapProvider::Okx.name() } else { name }))
    });
    let relay = relay::SUPPORTED_CHAINS.iter().filter(|asset| asset.get_chain().chain_type() == ChainType::Ethereum).flat_map(|asset| {
        let chain = asset.get_chain();
        let contracts = relay::contracts(chain);
        contracts.targets.into_iter().chain(contracts.spenders).map(move |address| ScanAddress::contract(chain, address, SwapProvider::Relay.name()))
    });
    let bridgers = bridgers::routers()
        .filter(|(chain, _)| chain.chain_type() == ChainType::Ethereum)
        .map(|(chain, address)| ScanAddress::contract(chain, address, SwapProvider::Bridgers.name()));
    let mayan_forwarders = mayan::supported_assets()
        .into_iter()
        .filter(|asset| asset.get_chain().chain_type() == ChainType::Ethereum)
        .map(|asset| ScanAddress::contract(asset.get_chain(), mayan::MAYAN_FORWARDER, SwapProvider::Mayan.name()));
    let sunswap = [(TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT, "SunSwap Router"), (TRON_SUNSWAP_PERMIT2_CONTRACT, "SunSwap Permit2")]
        .into_iter()
        .map(|(address, name)| ScanAddress::contract(Chain::Tron, address, name));

    routers.chain(contracts).chain(treasuries).chain(okx).chain(relay).chain(bridgers).chain(mayan_forwarders).chain(sunswap).collect()
}

fn permit2_name(provider: SwapProvider, address: &str) -> String {
    match address {
        UNISWAP_PERMIT2_CONTRACT | ZKSYNC_UNISWAP_PERMIT2_CONTRACT => "Permit2".to_string(),
        _ => format!("{} Permit2", provider.name()),
    }
}

#[cfg(test)]
mod tests {
    use gem_evm::ethereum_address_checksum;

    use super::*;

    #[test]
    fn test_approval_contracts_are_identified_with_generic_provider_names() {
        let contracts = known_contracts();
        let relay = relay::contracts(Chain::Ethereum);
        let (_, bridgers) = bridgers::routers().find(|(chain, _)| *chain == Chain::SmartChain).unwrap();
        let entries = [
            (Chain::Ethereum, relay.spenders[1], "Relay"),
            (Chain::SmartChain, bridgers, "Bridgers"),
            (Chain::Base, mayan::MAYAN_FORWARDER, "Mayan"),
            (Chain::Tron, TRON_SUNSWAP_PERMIT2_CONTRACT, "SunSwap Permit2"),
            (Chain::Tron, TRON_SUNSWAP_UNIVERSAL_ROUTER_CONTRACT, "SunSwap Router"),
        ];

        for (chain, address, name) in entries {
            assert_eq!(contracts.iter().find(|contract| contract.chain == chain && contract.address == address), Some(&ScanAddress::contract(chain, address, name)));
        }
    }

    #[test]
    fn test_provider_identification_keeps_supported_chain_scope_and_checksums() {
        let contracts = known_contracts();
        let relay = relay::contracts(Chain::Fantom);
        let (_, bridgers) = bridgers::routers().find(|(chain, _)| *chain == Chain::SmartChain).unwrap();

        for (chain, address) in [
            (Chain::Fantom, relay.spenders[1]),
            (Chain::Ethereum, bridgers),
            (Chain::Fantom, mayan::MAYAN_FORWARDER),
            (Chain::Ethereum, TRON_SUNSWAP_PERMIT2_CONTRACT),
        ] {
            assert_eq!(contracts.iter().find(|contract| contract.chain == chain && contract.address == address), None);
        }
        for provider in [SwapProvider::Relay, SwapProvider::Bridgers, SwapProvider::Mayan] {
            for contract in contracts.iter().filter(|contract| contract.chain.chain_type() == ChainType::Ethereum && contract.name.as_deref() == Some(provider.name())) {
                assert_eq!(contract.address, ethereum_address_checksum(&contract.address).unwrap());
            }
        }
        for provider in [SwapProvider::Relay, SwapProvider::Bridgers] {
            let other_chains = contracts
                .iter()
                .filter(|contract| contract.chain.chain_type() != ChainType::Ethereum && contract.name.as_deref() == Some(provider.name()))
                .count();
            assert_eq!(other_chains, 0, "{}", provider.name());
        }
    }

    #[test]
    fn test_okx_approval_is_identified_on_its_deployed_chains() {
        let contracts = known_contracts();
        let address = "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f";

        for chain in [Chain::Ethereum, Chain::AvalancheC] {
            assert_eq!(
                contracts.iter().find(|contract| contract.chain == chain && contract.address == address),
                Some(&ScanAddress::contract(chain, address, "OKX DEX Approval"))
            );
        }
    }

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

    #[test]
    fn test_a_treasury_is_saved_as_a_named_address() {
        let contracts = known_contracts();

        for (chain, treasury) in near_intents::TREASURY_ADDRESSES {
            let row = contracts.iter().find(|contract| contract.chain == chain && contract.address == treasury).unwrap();
            assert_eq!(row.address_type, Some(AddressType::Address), "{chain} {treasury}");
            assert_eq!(row.name.as_deref(), Some("NEAR Intents"));
        }
    }
}
