use primitives::{Chain, ChainAddress, ChainType, WalletConnectCAIP2};

fn is_supported(chain: Chain) -> bool {
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

pub fn account_identifier(chain: Chain, address: &str) -> Option<String> {
    if !is_supported(chain) {
        return None;
    }
    let namespace = WalletConnectCAIP2::get_namespace(chain)?;
    let reference = WalletConnectCAIP2::get_reference(chain)?;
    Some(format!("{namespace}:{reference}:{address}"))
}

pub fn parse_account(account: &str) -> Option<ChainAddress> {
    let account = WalletConnectCAIP2::parse_account(account.to_string())?;
    is_supported(account.chain).then_some(account)
}

pub fn account_chain(account: &str) -> Option<Chain> {
    parse_account(account).map(|account| account.chain)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_identifier() {
        assert_eq!(account_identifier(Chain::Ethereum, "0x1"), Some("eip155:1:0x1".to_string()));
        assert_eq!(account_identifier(Chain::Base, "0x1"), Some("eip155:8453:0x1".to_string()));
        assert_eq!(account_identifier(Chain::Bitcoin, "bc1"), None);
        assert_eq!(account_identifier(Chain::Solana, "abc"), None);
        assert_eq!(account_identifier(Chain::Cosmos, "cosmos1"), None);
        assert_eq!(account_identifier(Chain::Ton, "UQA"), None);
        assert_eq!(account_identifier(Chain::Tron, "TX"), None);
    }

    #[test]
    fn test_account_chain() {
        assert_eq!(account_chain("eip155:1:0x1"), Some(Chain::Ethereum));
        assert_eq!(account_chain("eip155:8453:0x1"), Some(Chain::Base));
        assert_eq!(account_chain("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp:abc"), None);
        assert_eq!(account_chain("cosmos:cosmoshub-4:cosmos1"), None);
        assert_eq!(account_chain("not-an-account"), None);
    }
}
