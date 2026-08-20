use super::{UniversalRouterAbi, get_uniswap_permit2_by_chain};
use primitives::{
    Chain,
    contract_constants::{OPTIMISM_UNISWAP_V4_QUOTER_CONTRACT, UNICHAIN_UNISWAP_V4_QUOTER_CONTRACT, UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT},
};

pub struct V4Deployment {
    pub quoter: &'static str, // V4 Quoter
    pub state_view: &'static str,
    pub permit2: &'static str,
    pub universal_router: &'static str,
    pub universal_router_abi: UniversalRouterAbi,
}

impl V4Deployment {
    fn v2_1(quoter: &'static str, state_view: &'static str, permit2: &'static str, universal_router: &'static str) -> Self {
        Self {
            quoter,
            state_view,
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
            "0x52F0E24D1c21C8A0cB1e5a5dD6198556BD9E1203",
            "0x7fFE42C4a5DEeA5b0feC41C94C136Cf115597227",
            permit2,
            "0x4C82D1fBFe28C977cBB58D8C7FF8FCF9F70a2cCA",
        )),
        Chain::Optimism => Some(V4Deployment::v2_1(
            OPTIMISM_UNISWAP_V4_QUOTER_CONTRACT,
            "0xc18a3169788F4F75A170290584ECA6395C75Ecdb",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Arbitrum => Some(V4Deployment::v2_1(
            "0x3972C00f7ed4885e145823eb7C655375d275A1C5",
            "0x76Fd297e2D437cd7f76d50F01AfE6160f86e9990",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Polygon => Some(V4Deployment::v2_1(
            "0xb3d5c3Dfc3a7aEbFF71895A7191796BFFc2c81b9",
            "0x5eA1bD7974c8A611cBAB0bDCAFcB1D9CC9b3BA5a",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::AvalancheC => Some(V4Deployment::v2_1(
            "0xbE40675BB704506a3c2Ccfb762DCFd1e979845C2",
            "0xc3c9e198C735a4b97e3e683f391cCBDD60B69286",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Base => Some(V4Deployment::v2_1(
            "0x0d5e0F971ED27FBfF6c2837bf31316121532048D",
            "0xA3c0c9b65baD0b08107Aa264b0f3dB444b867A71",
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::SmartChain => Some(V4Deployment::v2_1(
            "0x9F75dD27D6664c475B90e105573E550ff69437B0",
            "0xd13Dd3D6E93f276FAfc9Db9E6BB47C1180aeE0c4",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Blast => Some(V4Deployment::v2_1(
            "0x6F71Cdcb0d119fF72C6eb501ABCEb576fBF62BCF",
            "0x12a88AE16F46DCe4e8B15368008Ab3380885df30",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Linea => Some(V4Deployment::v2_1(
            "0x2C125569C0BeE20A66E33E5491C552B37EBD9934",
            "0xE861de206E460A8b936b05ad3816520B58ccDf9b",
            permit2,
            "0xBA548cE7A95f87Bc66a0C7c6eAB1e428735F8b57",
        )),
        Chain::World => Some(V4Deployment::v2_1(
            "0x55d235b3fF2DaF7c3ede0defC9521f1d6Fe6c5c0",
            "0x51D394718bc09297262e368c1A481217FdEB71eb",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Unichain => Some(V4Deployment::v2_1(
            UNICHAIN_UNISWAP_V4_QUOTER_CONTRACT,
            "0x86e8631A016F9068C3f085fAF484Ee3F5fDee8f2",
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::Celo => Some(V4Deployment::v2_1(
            "0x28566da1093609182dFf2cB2A91CFD72e61d66cd",
            "0xbc21f8720BABf4b20d195eE5C6e99c52b76F2bfb",
            permit2,
            "0x8B844f885672f333Bc0042cB669255f93a4C1E6b",
        )),
        Chain::Monad => Some(V4Deployment::v2_1(
            "0xa222Dd357A9076d1091Ed6Aa2e16C9742dD26891",
            "0x77395F3b2E73aE90843717371294fa97cC419D64",
            permit2,
            "0xFdf682F51FE81Aa4898F0AE2163d8A55c127fbC7",
        )),
        Chain::Ink => Some(V4Deployment::v2_1(
            "0x3972C00f7ed4885e145823eb7C655375d275A1C5",
            "0x76Fd297e2D437cd7f76d50F01AfE6160f86e9990",
            permit2,
            "0x28bD21bB4Ea4fDa370D8d7544992038375D8d456",
        )),
        Chain::XLayer => Some(V4Deployment::v2_1(
            "0x8928074CA1b241D8Ec02815881c1Af11E8bC5219",
            "0x76Fd297e2D437cd7f76d50F01AfE6160f86e9990",
            permit2,
            "0xDa00aE15d3A71466517129255255db7c0c0956d3",
        )),
        // See: https://github.com/Uniswap/contracts/blob/main/deployments/4663.md
        Chain::Robinhood => Some(V4Deployment::v2_1(
            "0x8Dc178eFB8111BB0973Dd9d722ebeFF267c98F94",
            "0xF3334192D15450CdD385c8B70e03f9A6bD9E673b",
            permit2,
            "0x8876789976dEcBfCbBbe364623C63652db8C0904",
        )),
        Chain::Tempo => Some(V4Deployment::v2_1(
            "0x20E6487C371a2086F841eF453F85378223DF4f4E",
            "0x21B954fBa3F5ddEbe77Ef2D47A3100c066908B2A",
            permit2,
            "0xA2Dc7d0266f0CC50b3eEaF36c9BFCeCFF1BEea91",
        )),
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
        Chain::Ethereum => "0x66a9893cC07D91D95644AEDD05D03f95e1dBA8Af",
        Chain::Optimism => "0x851116D9223fabED8E56C0E6b8Ad0c31d98B3507",
        Chain::Arbitrum => "0xA51afAFe0263b40EdaEf0Df8781eA9aa03E381a3",
        Chain::Polygon => "0x1095692A6237d83C6a72F3F5eFEdb9A670C49223",
        Chain::AvalancheC => "0x94b75331AE8d42C1b61065089B7d48FE14aA73b7",
        Chain::Base => "0x6fF5693b99212Da76ad316178A184AB56D299b43",
        Chain::SmartChain => "0x1906c1d672b88cD1B9aC7593301cA990F94Eae07",
        Chain::Blast => "0xeAbBcB3E8E415306207ef514f660A3F820025BE3",
        Chain::Linea => "0x661E93cca42AfacB172121EF892830cA3b70F08d",
        Chain::World => "0x8ac7bEE993bb44dAb564Ea4bc9EA67Bf9Eb5e743",
        Chain::Unichain => UNICHAIN_UNISWAP_V4_UNIVERSAL_ROUTER_CONTRACT,
        Chain::Celo => "0xcb695bc5D3Aa22cAD1E6DF07801b061a05A0233A",
        Chain::Monad => "0x0D97Dc33264bfC1c226207428A79b26757fb9dc3",
        Chain::Ink => "0x112908daC86e20e7241B0927479Ea3Bf935d1fa0",
        Chain::XLayer => "0x5507749F2c558Bb3E162c6e90c314c092E7372Ff",
        _ => return None,
    };

    legacy_router.eq_ignore_ascii_case(contract).then_some(UniversalRouterAbi::V2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;
    use primitives::contract_constants::UNISWAP_PERMIT2_CONTRACT;
    use std::str::FromStr;

    #[test]
    fn test_deployment_addresses_are_checksummed() {
        for chain in Chain::all() {
            let Some(deployment) = get_uniswap_deployment_by_chain(&chain) else {
                continue;
            };
            for value in [deployment.quoter, deployment.state_view, deployment.permit2, deployment.universal_router] {
                let address = Address::from_str(value).unwrap();
                assert_eq!(value, address.to_checksum(None), "{chain} deployment address");
            }
        }
    }

    #[test]
    fn test_robinhood_uniswap_v4_deployment() {
        let deployment = get_uniswap_deployment_by_chain(&Chain::Robinhood).unwrap();

        assert_eq!(deployment.quoter, "0x8Dc178eFB8111BB0973Dd9d722ebeFF267c98F94");
        assert_eq!(deployment.state_view, "0xF3334192D15450CdD385c8B70e03f9A6bD9E673b");
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
