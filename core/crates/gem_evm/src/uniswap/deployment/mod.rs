pub mod aero;
pub mod v2;
pub mod v3;
pub mod v4;

use primitives::contract_constants::{UNISWAP_PERMIT2_CONTRACT, ZKSYNC_UNISWAP_PERMIT2_CONTRACT};
use primitives::{Chain, SwapProvider};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UniversalRouterAbi {
    V2,
    V2_1,
}

pub fn get_uniswap_permit2_by_chain(chain: &Chain) -> Option<&'static str> {
    match chain {
        Chain::Ethereum
        | Chain::Optimism
        | Chain::Arbitrum
        | Chain::Polygon
        | Chain::AvalancheC
        | Chain::Base
        | Chain::SmartChain
        | Chain::Celo
        | Chain::Blast
        | Chain::Linea
        | Chain::World
        | Chain::Unichain
        | Chain::Ink
        | Chain::Monad
        | Chain::XLayer
        | Chain::Stable
        | Chain::Robinhood
        | Chain::Tempo
        | Chain::Arc => Some(UNISWAP_PERMIT2_CONTRACT),
        Chain::ZkSync | Chain::Abstract => Some(ZKSYNC_UNISWAP_PERMIT2_CONTRACT),
        _ => None,
    }
}

pub fn is_permit2_contract_by_chain(chain: &Chain, contract: &str) -> bool {
    [
        Some(UNISWAP_PERMIT2_CONTRACT),
        get_uniswap_permit2_by_chain(chain),
        v3::get_pancakeswap_router_deployment_by_chain(chain).map(|deployment| deployment.permit2),
        v3::get_oku_deployment_by_chain(chain).map(|deployment| deployment.permit2),
        v3::get_wagmi_router_deployment_by_chain(chain).map(|deployment| deployment.permit2),
    ]
    .into_iter()
    .flatten()
    .any(|permit2| permit2.eq_ignore_ascii_case(contract))
}

pub fn get_provider_by_chain_contract(chain: &Chain, contract: &str) -> Option<SwapProvider> {
    if v3::is_uniswap_router_contract_by_chain(chain, contract) {
        return Some(SwapProvider::UniswapV3);
    }
    if v4::is_uniswap_router_contract_by_chain(chain, contract) {
        return Some(SwapProvider::UniswapV4);
    }
    [
        (v3::get_pancakeswap_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router), SwapProvider::PancakeswapV3),
        (v3::get_oku_deployment_by_chain(chain).map(|deployment| deployment.universal_router), SwapProvider::Oku),
        (v3::get_wagmi_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router), SwapProvider::Wagmi),
        (v3::get_aerodrome_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router), SwapProvider::Aerodrome),
    ]
    .into_iter()
    .find_map(|(router, provider)| router.filter(|router| router.eq_ignore_ascii_case(contract)).map(|_| provider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_permit2_contract_by_chain() {
        for chain in Chain::all() {
            let v3_deployments = [
                v3::get_uniswap_router_deployment_by_chain(&chain),
                v3::get_pancakeswap_router_deployment_by_chain(&chain),
                v3::get_oku_deployment_by_chain(&chain),
                v3::get_wagmi_router_deployment_by_chain(&chain),
                v3::get_aerodrome_router_deployment_by_chain(&chain),
            ];
            let permit2_contracts = v3_deployments
                .into_iter()
                .flatten()
                .map(|deployment| deployment.permit2)
                .chain(v4::get_uniswap_deployment_by_chain(&chain).map(|deployment| deployment.permit2));
            for permit2 in permit2_contracts {
                assert!(is_permit2_contract_by_chain(&chain, permit2), "{chain} {permit2}");
                assert!(is_permit2_contract_by_chain(&chain, &permit2.to_lowercase()), "{chain} {permit2}");
            }
            assert!(is_permit2_contract_by_chain(&chain, UNISWAP_PERMIT2_CONTRACT), "{chain}");
        }

        let pancakeswap_permit2 = v3::get_pancakeswap_router_deployment_by_chain(&Chain::SmartChain).unwrap().permit2;
        assert!(!is_permit2_contract_by_chain(&Chain::Polygon, pancakeswap_permit2));
        assert!(!is_permit2_contract_by_chain(&Chain::Ethereum, ZKSYNC_UNISWAP_PERMIT2_CONTRACT));
        assert!(!is_permit2_contract_by_chain(&Chain::Ethereum, "0x2222222222222222222222222222222222222222"));
    }
}
