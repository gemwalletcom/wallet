use crate::{Contracts, Deployment};
use primitives::{Chain, contract_constants::SOLANA_OKX_DEX_V2_PROGRAM_ID};

pub(super) const PROXY_QUOTE_PATH: &str = "/v6/quote";
pub(super) const PROXY_SWAP_PATH: &str = "/v6/swap";

const DEFAULT_EVM_GAS_LIMIT: u64 = 920_000;

const SOLANA_DEX_IDS: &str = "277,278,279,343,72,103,284,338,372,403,444,483,357,345,459,457,475,342";

const DEPLOYMENTS: &[Deployment] = &[
    Deployment::new(Chain::Solana, SOLANA_OKX_DEX_V2_PROGRAM_ID),
    Deployment::with_spender(Chain::Ethereum, "0x8feAB81D36E7576107D5dE0758c1b839Be31B4F6", "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f"),
    Deployment::with_spender(Chain::SmartChain, "0x5994814f2C4040b863A0125A45DE152a8c2A4DEc", "0x2c34A2Fb1d0b4f55de51E1d0bDEfaDDce6b7cDD6"),
    Deployment::with_spender(Chain::Polygon, "0x3C4829196BFadFF4394726b45159aeaAC6FCd41c", "0x3B86917369B83a6892f553609F3c2F439C184e31"),
    Deployment::with_spender(Chain::Arbitrum, "0x09f94b5Fc68e227C323A6FbaE3Bd98C97fD8c849", "0x70cBb871E8f30Fc8Ce23609E9E0Ea87B6b222F58"),
    Deployment::with_spender(Chain::Optimism, "0x1f5B43127414E36c31eCb5Ff5567262997CD24D0", "0x68D6B739D2020067D1e2F713b999dA97E4d54812"),
    Deployment::with_spender(Chain::Base, "0x67d03631FE51B741C0C00c4E16eb662AC84381df", "0x57df6092665eb6058DE53939612413ff4B09114E"),
    Deployment::with_spender(Chain::AvalancheC, "0xAB96dcFA7A7D669d9BF5918faB8641479973dD0A", "0x40aA958dd87FC8305b97f2BA922CDdCa374bcD7f"),
    Deployment::with_spender(Chain::Fantom, "0xd72f9Af181A0eB1B8550a00124ECdb71Bb758C89", "0x70cBb871E8f30Fc8Ce23609E9E0Ea87B6b222F58"),
    Deployment::with_spender(Chain::Manta, "0xd72f9Af181A0eB1B8550a00124ECdb71Bb758C89", "0x57df6092665eb6058DE53939612413ff4B09114E"),
    Deployment::with_spender(Chain::ZkSync, "0x46eDEcEa0228f04Ab88dC34BE98314863bA40bE0", "0xc67879F4065d3B9fe1C09EE990B891Aa8E3a4c2f"),
    Deployment::with_spender(Chain::Linea, "0xdfcb0cEcC10e78F3F3749F3f3d3EE4047b2c9829", "0x57df6092665eb6058DE53939612413ff4B09114E"),
    Deployment::with_spender(Chain::Mantle, "0x472fc4f7fd3C9F06f0b8637c5505815ac80938Ad", "0x57df6092665eb6058DE53939612413ff4B09114E"),
    Deployment::with_spender(Chain::Plasma, "0xd72f9Af181A0eB1B8550a00124ECdb71Bb758C89", "0x9FD43F5E4c24543b2eBC807321E58e6D350d6a5A"),
    Deployment::with_spender(Chain::Hyperliquid, "0xb193874f0C77948d2bCFeC2EfAF8Bc65B4C2ca89", "0x56e6983D59bF472Ced0E63966A14d94A3A291589"),
    Deployment::with_spender(Chain::Sonic, "0xd72f9Af181A0eB1B8550a00124ECdb71Bb758C89", "0xD321ab5589d3E8FA5Df985ccFEf625022E2DD910"),
    Deployment::with_spender(Chain::Unichain, "0xe3dAb8Bf5187F9B4E8E89FF5414D7CF71E2C82e1", "0x2e28281Cf3D58f475cebE27bec4B8a23dFC7782c"),
    Deployment::with_spender(Chain::Monad, "0xc1C76E784Db8D68585fb608ce68FC5DcFF14000E", "0xf534A8a1CAD0543Cd6438f7534CA3486c01998d4"),
    Deployment::with_spender(Chain::XLayer, "0x7c5bEE2a8091C3ef39072f64F18Fac913060AEaF", "0x8b773D83bc66Be128c60e07E17C8901f7a64F000"),
    Deployment::with_spender(Chain::Robinhood, "0x6e2A35A7AD683cF634D91492d73bb7FF774c6919", "0x42170295F1173c9e5874ea9d00c6d137E1a4f53d"),
];

pub(super) fn supported_chains() -> impl Iterator<Item = Chain> {
    DEPLOYMENTS.iter().map(|deployment| deployment.chain)
}

pub(super) fn chain_index(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some("501"),
        _ => supported_chains().any(|supported| supported == chain).then(|| chain.config().network_id),
    }
}

pub(super) fn contracts(chain: Chain) -> Contracts {
    Deployment::contracts(DEPLOYMENTS, chain)
}

pub(super) fn dex_ids(chain: Chain) -> Option<&'static str> {
    match chain {
        Chain::Solana => Some(SOLANA_DEX_IDS),
        _ => None,
    }
}

pub(super) fn evm_gas_limit(chain: Chain) -> u64 {
    match chain {
        Chain::Manta => 600_000,
        Chain::ZkSync => 2_000_000,
        Chain::Mantle => 2_000_000_000,
        _ => DEFAULT_EVM_GAS_LIMIT,
    }
}
