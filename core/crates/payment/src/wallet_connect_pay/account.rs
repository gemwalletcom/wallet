use primitives::{Chain, ChainAddress, ChainType, WalletConnectCAIP2};

pub fn is_supported(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ethereum => true,
        ChainType::Bitcoin
        | ChainType::Solana
        | ChainType::Cosmos
        | ChainType::Ton
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Sui
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => false,
    }
}

pub fn get_account_identifier(chain: Chain, address: &str) -> Option<String> {
    let namespace = WalletConnectCAIP2::get_namespace(chain)?;
    let reference = WalletConnectCAIP2::get_reference(chain)?;
    Some(format!("{namespace}:{reference}:{address}"))
}

pub fn get_account(account: &str) -> Option<ChainAddress> {
    WalletConnectCAIP2::parse_account(account.to_string())
}

pub fn get_chain(account: &str) -> Option<Chain> {
    get_account(account).map(|account| account.chain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported() {
        assert!(is_supported(Chain::Ethereum));
        assert!(is_supported(Chain::Base));

        assert!(!is_supported(Chain::Bitcoin));
        assert!(!is_supported(Chain::Solana));
        assert!(!is_supported(Chain::Cosmos));
        assert!(!is_supported(Chain::Ton));
        assert!(!is_supported(Chain::Tron));
    }

    #[test]
    fn test_get_account_identifier() {
        assert_eq!(get_account_identifier(Chain::Ethereum, "0x1"), Some("eip155:1:0x1".to_string()));
        assert_eq!(get_account_identifier(Chain::Base, "0x1"), Some("eip155:8453:0x1".to_string()));
        assert_eq!(
            get_account_identifier(Chain::Solana, "abc"),
            Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:abc".to_string())
        );
        assert_eq!(get_account_identifier(Chain::Bitcoin, "bc1"), None);
    }

    #[test]
    fn test_get_chain() {
        assert_eq!(get_chain("eip155:1:0x1"), Some(Chain::Ethereum));
        assert_eq!(get_chain("eip155:8453:0x1"), Some(Chain::Base));
        assert_eq!(get_chain("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:abc"), Some(Chain::Solana));
        assert_eq!(get_chain("eip155:99999:0x1"), None);
        assert_eq!(get_chain("not-an-account"), None);
    }
}
