use super::{UniversalRouterAbi, get_uniswap_permit2_by_chain};
use primitives::{
    Chain,
    contract_constants::{OPTIMISM_UNISWAP_V4_QUOTER_CONTRACT, UNICHAIN_UNISWAP_V4_QUOTER_CONTRACT, UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT},
};

pub struct V4Deployment {
    pub quoter: &'static str, // V4 Quoter
    pub permit2: &'static str,
    pub universal_router: &'static str,
    pub universal_router_abi: UniversalRouterAbi,
}

impl V4Deployment {
    fn v2_1(quoter: &'static str, permit2: &'static str, universal_router: &'static str) -> Self {
        Self {
            quoter,
            permit2,
            universal_router,
            universal_router_abi: UniversalRouterAbi::V2_1,
        }
    }
}

pub fn get_uniswap_deployment_by_chain(chain: &Chain) -> Option<V4Deployment> {
    // https://github.com/Uniswap/contracts/blob/main/deployments/index.md
    let permit2 = get_uniswap_permit2_by_chain(chain)?;
    match chain {
        Chain::Ethereum => Some(V4Deployment::v2_1(
            "0x52f0e24d1c21c8a0cb1e5a5dd6198556bd9e1203",
            permit2,
            "0x4C82D1fBFe28C977cBB58D8C7FF8FCF9F70a2cCA",
        )),
        Chain::Optimism => Some(V4Deployment::v2_1(
            OPTIMISM_UNISWAP_V4_QUOTER_CONTRACT,
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Arbitrum => Some(V4Deployment::v2_1(
            "0x3972c00f7ed4885e145823eb7c655375d275a1c5",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Polygon => Some(V4Deployment::v2_1(
            "0xb3d5c3dfc3a7aebff71895a7191796bffc2c81b9",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::AvalancheC => Some(V4Deployment::v2_1(
            "0xbe40675bb704506a3c2ccfb762dcfd1e979845c2",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Base => Some(V4Deployment::v2_1(
            "0x0d5e0f971ed27fbff6c2837bf31316121532048d",
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::SmartChain => Some(V4Deployment::v2_1(
            "0x9f75dd27d6664c475b90e105573e550ff69437b0",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Blast => Some(V4Deployment::v2_1(
            "0x6f71cdcb0d119ff72c6eb501abceb576fbf62bcf",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::World => Some(V4Deployment::v2_1(
            "0x55d235b3ff2daf7c3ede0defc9521f1d6fe6c5c0",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Unichain => Some(V4Deployment::v2_1(
            UNICHAIN_UNISWAP_V4_QUOTER_CONTRACT,
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::Celo => Some(V4Deployment::v2_1(
            "0x28566da1093609182dFf2cB2A91CFD72e61d66cd",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Monad => Some(V4Deployment::v2_1(
            "0xa222Dd357A9076d1091Ed6Aa2e16C9742dD26891",
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::Ink => Some(V4Deployment::v2_1(
            "0x3972C00f7ed4885e145823eb7C655375d275A1C5",
            permit2,
            "0x28bD21bB4Ea4fDa370D8d7544992038375D8d456",
        )),
        // See: https://github.com/Uniswap/contracts/blob/main/deployments/4663.md
        Chain::Robinhood => Some(V4Deployment::v2_1(
            "0x8Dc178eFB8111BB0973Dd9d722ebeFF267c98F94",
            permit2,
            "0x8876789976dEcBfCbBbe364623C63652db8C0904",
        )),
        // Chain::XLayer => Some(V4Deployment {
        //     quoter: "0x8928074CA1b241D8Ec02815881c1Af11E8bC5219",
        //     permit2,
        //     universal_router: "0xDa00aE15d3A71466517129255255db7c0c0956d3",
        // }),
        _ => None,
    }
}

pub fn get_universal_router_abi_by_chain_contract(chain: &Chain, contract: &str) -> Option<UniversalRouterAbi> {
    if let Some(deployment) = get_uniswap_deployment_by_chain(chain)
        && deployment.universal_router.eq_ignore_ascii_case(contract)
    {
        return Some(deployment.universal_router_abi);
    }

    legacy_uniswap_router_abi_by_chain_contract(chain, contract)
}

pub fn is_uniswap_router_contract_by_chain(chain: &Chain, contract: &str) -> bool {
    get_universal_router_abi_by_chain_contract(chain, contract).is_some()
}

fn legacy_uniswap_router_abi_by_chain_contract(chain: &Chain, contract: &str) -> Option<UniversalRouterAbi> {
    let legacy_router = match chain {
        Chain::Ethereum => "0x66a9893cc07d91d95644aedd05d03f95e1dba8af",
        Chain::Optimism => "0x851116d9223fabed8e56c0e6b8ad0c31d98b3507",
        Chain::Arbitrum => "0xa51afafe0263b40edaef0df8781ea9aa03e381a3",
        Chain::Polygon => "0x1095692a6237d83c6a72f3f5efedb9a670c49223",
        Chain::AvalancheC => "0x94b75331ae8d42c1b61065089b7d48fe14aa73b7",
        Chain::Base => "0x6ff5693b99212da76ad316178a184ab56d299b43",
        Chain::SmartChain => "0x1906c1d672b88cd1b9ac7593301ca990f94eae07",
        Chain::Blast => "0xeabbcb3e8e415306207ef514f660a3f820025be3",
        Chain::World => "0x8ac7bee993bb44dab564ea4bc9ea67bf9eb5e743",
        Chain::Unichain => UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT,
        Chain::Celo => "0xcb695bc5D3Aa22cAD1E6DF07801b061a05A0233A",
        Chain::Monad => "0x0D97Dc33264bfC1c226207428A79b26757fb9dc3",
        Chain::Ink => "0x112908daC86e20e7241B0927479Ea3Bf935d1fa0",
        _ => return None,
    };

    legacy_router.eq_ignore_ascii_case(contract).then_some(UniversalRouterAbi::V2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;

    #[test]
    fn test_robinhood_uniswap_v4_deployment() {
        let deployment = get_uniswap_deployment_by_chain(&Chain::Robinhood).unwrap();

        assert_eq!(deployment.quoter, "0x8Dc178eFB8111BB0973Dd9d722ebeFF267c98F94");
        assert_eq!(deployment.permit2, UNISWAP_PERMIT2_CONTRACT);
        assert_eq!(deployment.universal_router, "0x8876789976dEcBfCbBbe364623C63652db8C0904");
        assert_eq!(deployment.universal_router_abi, UniversalRouterAbi::V2_1);
    }

    #[test]
    fn test_uniswap_v4_v2_1_deployments() {
        let ethereum = get_uniswap_deployment_by_chain(&Chain::Ethereum).unwrap();
        let base = get_uniswap_deployment_by_chain(&Chain::Base).unwrap();
        let optimism = get_uniswap_deployment_by_chain(&Chain::Optimism).unwrap();

        assert_eq!(ethereum.universal_router, "0x4C82D1fBFe28C977cBB58D8C7FF8FCF9F70a2cCA");
        assert_eq!(base.universal_router, "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7");
        assert_eq!(optimism.universal_router, "0x8B844f885672f333Bc0042cB669255f93a4C1E6b");
        assert_eq!(ethereum.universal_router_abi, UniversalRouterAbi::V2_1);
        assert_eq!(base.universal_router_abi, UniversalRouterAbi::V2_1);
        assert_eq!(optimism.universal_router_abi, UniversalRouterAbi::V2_1);
    }

    #[test]
    fn test_uniswap_v4_ink_deployment() {
        let ink = get_uniswap_deployment_by_chain(&Chain::Ink).unwrap();

        assert_eq!(ink.universal_router, "0x28bD21bB4Ea4fDa370D8d7544992038375D8d456");
        assert_eq!(ink.universal_router_abi, UniversalRouterAbi::V2_1);
        assert_eq!(
            get_universal_router_abi_by_chain_contract(&Chain::Ink, "0x112908daC86e20e7241B0927479Ea3Bf935d1fa0"),
            Some(UniversalRouterAbi::V2)
        );
    }
}
