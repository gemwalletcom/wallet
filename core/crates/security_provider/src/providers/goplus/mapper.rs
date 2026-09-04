use primitives::Chain;

pub fn map_address_chain(chain: Chain) -> Result<&'static str, String> {
    match chain {
        Chain::Ethereum => Ok("1"),
        Chain::SmartChain => Ok("56"),
        Chain::Polygon => Ok("137"),
        Chain::Arbitrum => Ok("42161"),
        Chain::Tron => Ok("tron"),
        Chain::Optimism => Ok("10"),
        Chain::Base => Ok("8453"),
        Chain::AvalancheC => Ok("43114"),
        Chain::OpBNB => Ok("204"),
        Chain::Gnosis => Ok("100"),
        Chain::ZkSync => Ok("324"),
        Chain::Linea => Ok("59144"),
        Chain::Robinhood => Ok("4663"),
        Chain::Stable => Ok("988"),
        _ => Err(format!("Unsupported GoPlus address chain: {chain}")),
    }
}

pub fn map_token_chain(chain: Chain) -> Result<&'static str, String> {
    match chain {
        Chain::Ethereum => Ok("1"),
        Chain::SmartChain => Ok("56"),
        Chain::Polygon => Ok("137"),
        Chain::Arbitrum => Ok("42161"),
        Chain::Tron => Ok("tron"),
        Chain::Optimism => Ok("10"),
        Chain::Base => Ok("8453"),
        Chain::AvalancheC => Ok("43114"),
        Chain::OpBNB => Ok("204"),
        Chain::Gnosis => Ok("100"),
        Chain::Manta => Ok("169"),
        Chain::Blast => Ok("81457"),
        Chain::ZkSync => Ok("324"),
        Chain::Linea => Ok("59144"),
        Chain::Mantle => Ok("5000"),
        Chain::World => Ok("480"),
        Chain::Sonic => Ok("146"),
        Chain::Plasma => Ok("9745"),
        Chain::Abstract => Ok("2741"),
        Chain::Berachain => Ok("80094"),
        Chain::Unichain => Ok("130"),
        Chain::Monad => Ok("143"),
        Chain::XLayer => Ok("196"),
        Chain::Robinhood => Ok("4663"),
        Chain::Stable => Ok("988"),
        _ => Err(format!("Unsupported GoPlus token chain: {chain}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maps_supported_chains() {
        assert_eq!(map_address_chain(Chain::Ethereum), Ok("1"));
        assert_eq!(map_token_chain(Chain::Blast), Ok("81457"));
    }

    #[test]
    fn test_rejects_unsupported_chains() {
        assert!(map_address_chain(Chain::Solana).is_err());
        assert!(map_address_chain(Chain::Blast).is_err());
        assert!(map_token_chain(Chain::Fantom).is_err());
    }
}
