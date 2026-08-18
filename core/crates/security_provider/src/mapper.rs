use primitives::{Chain, EVMChain};

pub fn chain_to_provider_id(chain: Chain) -> String {
    match EVMChain::from_chain(chain) {
        Some(_) => chain.network_id().to_string(),
        // GoPlus and HashDit only screen EVM chains; default the rest to Ethereum
        None => "1".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_to_provider_id() {
        assert_eq!(chain_to_provider_id(Chain::Ethereum), "1");
        assert_eq!(chain_to_provider_id(Chain::SmartChain), "56");
        assert_eq!(chain_to_provider_id(Chain::Polygon), "137");
        assert_eq!(chain_to_provider_id(Chain::Arbitrum), "42161");
        assert_eq!(chain_to_provider_id(Chain::Optimism), "10");
        assert_eq!(chain_to_provider_id(Chain::Base), "8453");
        assert_eq!(chain_to_provider_id(Chain::Monad), "143");
        assert_eq!(chain_to_provider_id(Chain::Bitcoin), "1"); // Non-EVM, defaults to Ethereum
    }
}
