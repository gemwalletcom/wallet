use crate::model::Platform;
use primitives::Chain;

const COINMARKETCAP_PLATFORM_NAMES: &[(Chain, &str)] = &[
    (Chain::SmartChain, "BNB Smart Chain (BEP20)"),
    (Chain::OpBNB, "opBNB"),
    (Chain::AvalancheC, "Avalanche C-Chain"),
    (Chain::Tron, "Tron20"),
    (Chain::Gnosis, "Gnosis Chain"),
    (Chain::ZkSync, "zkSync Era"),
    (Chain::Manta, "Manta Pacific"),
    (Chain::SeiEvm, "Sei v2"),
    (Chain::Sei, "Sei Network"),
    (Chain::XLayer, "X Layer"),
];

const COINMARKETCAP_PLATFORM_SLUGS: &[(Chain, &str)] = &[
    (Chain::Ethereum, "ethereum"),
    (Chain::SmartChain, "bnb"),
    (Chain::Polygon, "polygon"),
    (Chain::Solana, "solana"),
    (Chain::Arbitrum, "arbitrum"),
    (Chain::Optimism, "optimism-ethereum"),
    (Chain::Base, "base"),
    (Chain::AvalancheC, "avalanche"),
    (Chain::Fantom, "fantom"),
    (Chain::Gnosis, "gnosis-gno"),
    (Chain::Tron, "tron"),
    (Chain::ZkSync, "zksync"),
    (Chain::Linea, "linea"),
    (Chain::Mantle, "mantle"),
    (Chain::Celo, "celo"),
    (Chain::Near, "near-protocol"),
    (Chain::Ton, "toncoin"),
    (Chain::Ton, "the-open-network"),
    (Chain::Ton, "gram"),
    (Chain::Sui, "sui"),
    (Chain::Aptos, "aptos"),
    (Chain::Algorand, "algorand"),
    (Chain::Stellar, "stellar"),
    (Chain::Sei, "sei"),
    (Chain::Injective, "injective"),
    (Chain::Osmosis, "osmosis"),
    (Chain::Manta, "manta-network"),
    (Chain::Blast, "blast"),
    (Chain::Sonic, "sonic"),
    (Chain::Berachain, "berachain"),
    (Chain::Ink, "ink"),
    (Chain::Unichain, "unichain"),
    (Chain::Monad, "monad"),
    (Chain::XLayer, "x-layer"),
    (Chain::XLayer, "okb"),
];

pub fn get_chain_for_coinmarketcap_platform(platform: &Platform) -> Option<Chain> {
    get_chain_for_coinmarketcap_platform_name(&platform.name).or_else(|| get_chain_for_coinmarketcap_platform_slug(&platform.coin.slug))
}

pub fn get_coinmarketcap_logo_url(logo: &str) -> Option<String> {
    Some(logo.replace("/64x64/", "/200x200/").replace("/128x128/", "/200x200/"))
}

fn get_chain_for_coinmarketcap_platform_slug(slug: &str) -> Option<Chain> {
    get_chain(COINMARKETCAP_PLATFORM_SLUGS, slug)
}

fn get_chain_for_coinmarketcap_platform_name(name: &str) -> Option<Chain> {
    get_chain(COINMARKETCAP_PLATFORM_NAMES, name)
}

fn get_chain(platforms: &[(Chain, &str)], id: &str) -> Option<Chain> {
    platforms.iter().find_map(|(chain, platform_id)| (*platform_id == id).then_some(*chain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Info;
    use serde_json::Value;

    #[test]
    fn test_coinmarketcap_platform_mapping_from_info_fixture() {
        let response: Value = serde_json::from_str(include_str!("../testdata/cryptocurrency_info.json")).unwrap();
        let eth: Info = serde_json::from_value(response["data"]["1027"].clone()).unwrap();

        let chains = eth
            .contract_address
            .iter()
            .filter_map(|contract| get_chain_for_coinmarketcap_platform(&contract.platform))
            .collect::<Vec<_>>();

        assert_eq!(chains, vec![Chain::Ethereum, Chain::SmartChain, Chain::OpBNB]);
    }

    #[test]
    fn test_coinmarketcap_platform_tables() {
        assert_eq!(get_chain_for_coinmarketcap_platform_name("BNB Smart Chain (BEP20)"), Some(Chain::SmartChain));
        assert_eq!(get_chain_for_coinmarketcap_platform_slug("bnb"), Some(Chain::SmartChain));
        assert_eq!(get_chain_for_coinmarketcap_platform_slug("unknown"), None);

        for (chain, name) in COINMARKETCAP_PLATFORM_NAMES {
            assert_eq!(get_chain_for_coinmarketcap_platform_name(name), Some(*chain));
        }
        for (chain, slug) in COINMARKETCAP_PLATFORM_SLUGS {
            assert_eq!(get_chain_for_coinmarketcap_platform_slug(slug), Some(*chain));
        }
    }

    #[test]
    fn test_coinmarketcap_logo_url_uses_largest_available_source() {
        assert_eq!(
            get_coinmarketcap_logo_url("https://s2.coinmarketcap.com/static/img/coins/64x64/1027.png"),
            Some("https://s2.coinmarketcap.com/static/img/coins/200x200/1027.png".to_string())
        );
        assert_eq!(
            get_coinmarketcap_logo_url("https://s2.coinmarketcap.com/static/img/coins/128x128/825.png"),
            Some("https://s2.coinmarketcap.com/static/img/coins/200x200/825.png".to_string())
        );
        assert_eq!(
            get_coinmarketcap_logo_url("https://s2.coinmarketcap.com/static/img/coins/200x200/825.png"),
            Some("https://s2.coinmarketcap.com/static/img/coins/200x200/825.png".to_string())
        );
    }
}
