use primitives::Chain;

pub enum AlchemyApi {
    JsonRpc,
    Nft,
}

pub fn alchemy_url(chain: Chain, base_url: &str, api: AlchemyApi, key: &str) -> String {
    let network = match chain {
        Chain::Ethereum => "eth-mainnet",
        Chain::SmartChain => "bnb-mainnet",
        Chain::Solana => "solana-mainnet",
        Chain::Polygon => "polygon-mainnet",
        Chain::Plasma => "plasma-mainnet",
        Chain::Arbitrum => "arb-mainnet",
        Chain::Optimism => "opt-mainnet",
        Chain::Base => "base-mainnet",
        Chain::AvalancheC => "avax-mainnet",
        Chain::OpBNB => "opbnb-mainnet",
        Chain::Gnosis => "gnosis-mainnet",
        Chain::Blast => "blast-mainnet",
        Chain::ZkSync => "zksync-mainnet",
        Chain::Linea => "linea-mainnet",
        Chain::Mantle => "mantle-mainnet",
        Chain::Celo => "celo-mainnet",
        Chain::World => "worldchain-mainnet",
        Chain::Sonic => "sonic-mainnet",
        Chain::SeiEvm => "sei-mainnet",
        Chain::Abstract => "abstract-mainnet",
        Chain::Berachain => "berachain-mainnet",
        Chain::Ink => "ink-mainnet",
        Chain::Unichain => "unichain-mainnet",
        Chain::Hyperliquid => "hyperliquid-mainnet",
        Chain::Monad => "monad-mainnet",
        Chain::XLayer => "xlayer-mainnet",
        Chain::Robinhood => "robinhood-mainnet",
        Chain::Stable => "stable-mainnet",
        Chain::Fantom => "fantom-mainnet",
        Chain::Manta => "manta-mainnet",
        _ => panic!("Alchemy is not supported for {chain}"),
    };
    let path = match api {
        AlchemyApi::JsonRpc => "v2",
        AlchemyApi::Nft => "nft/v3",
    };
    let base_url = base_url.replace("{chain}", chain.as_ref()).replace("{network}", network);
    format!("{base_url}/{path}/{key}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alchemy_url() {
        assert_eq!(
            alchemy_url(Chain::Solana, "https://alchemy.example/{chain}", AlchemyApi::JsonRpc, "key"),
            "https://alchemy.example/solana/v2/key"
        );
        assert_eq!(
            alchemy_url(Chain::Solana, "https://{network}.g.alchemy.com", AlchemyApi::JsonRpc, "key"),
            "https://solana-mainnet.g.alchemy.com/v2/key"
        );
        assert_eq!(
            alchemy_url(Chain::SmartChain, "https://{network}.g.alchemy.com", AlchemyApi::Nft, "key"),
            "https://bnb-mainnet.g.alchemy.com/nft/v3/key"
        );
    }
}
