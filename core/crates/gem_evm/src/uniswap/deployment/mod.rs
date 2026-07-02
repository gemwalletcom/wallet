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
        | Chain::World
        | Chain::Unichain
        | Chain::Ink
        | Chain::Monad
        | Chain::XLayer
        | Chain::Stable
        | Chain::Robinhood => Some(UNISWAP_PERMIT2_CONTRACT),
        Chain::ZkSync | Chain::Abstract => Some(ZKSYNC_UNISWAP_PERMIT2_CONTRACT),
        _ => None,
    }
}

pub fn get_provider_by_chain_contract(chain: &Chain, contract: &str) -> Option<String> {
    if v3::is_uniswap_router_contract_by_chain(chain, contract) {
        return Some(SwapProvider::UniswapV3.id().to_string());
    }
    if v4::is_uniswap_router_contract_by_chain(chain, contract) {
        return Some(SwapProvider::UniswapV4.id().to_string());
    }
    [
        (
            v3::get_pancakeswap_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router),
            SwapProvider::PancakeswapV3,
        ),
        (v3::get_oku_deployment_by_chain(chain).map(|deployment| deployment.universal_router), SwapProvider::Oku),
        (
            v3::get_wagmi_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router),
            SwapProvider::Wagmi,
        ),
        (
            v3::get_aerodrome_router_deployment_by_chain(chain).map(|deployment| deployment.universal_router),
            SwapProvider::Aerodrome,
        ),
    ]
    .into_iter()
    .find_map(|(router, provider)| router.filter(|router| router.eq_ignore_ascii_case(contract)).map(|_| provider.id().to_string()))
}
