use primitives::Chain;

pub fn map_chain(chain: Chain) -> Result<&'static str, String> {
    match chain {
        Chain::Ethereum => Ok("1"),
        Chain::SmartChain => Ok("56"),
        Chain::Polygon => Ok("137"),
        Chain::Arbitrum => Ok("42161"),
        Chain::Optimism => Ok("10"),
        Chain::Base => Ok("8453"),
        Chain::AvalancheC => Ok("43114"),
        Chain::OpBNB => Ok("204"),
        Chain::ZkSync => Ok("324"),
        Chain::Linea => Ok("59144"),
        Chain::Mantle => Ok("5000"),
        Chain::Sonic => Ok("146"),
        Chain::Abstract => Ok("2741"),
        Chain::Berachain => Ok("80094"),
        Chain::Monad => Ok("143"),
        Chain::Robinhood => Ok("4663"),
        _ => Err(format!("Unsupported HashDit chain: {chain}")),
    }
}

pub fn map_poisoning_chain(chain: Chain) -> Result<&'static str, String> {
    match chain {
        Chain::Tron => Ok("tron"),
        _ => map_chain(chain),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maps_supported_chain() {
        assert_eq!(map_chain(Chain::SmartChain), Ok("56"));
    }

    #[test]
    fn test_rejects_unsupported_chain() {
        assert!(map_chain(Chain::Gnosis).is_err());
    }
}
