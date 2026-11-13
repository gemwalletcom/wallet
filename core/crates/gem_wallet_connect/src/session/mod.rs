mod sui;
mod ton;
mod tron;

use std::collections::HashMap;

use primitives::{Account, Chain, ChainType};

use self::{sui::sui_session_properties, ton::ton_session_properties, tron::tron_session_properties};

pub fn config_session_properties(mut properties: HashMap<String, String>, chains: &[Chain], accounts: &[Account]) -> HashMap<String, String> {
    for chain in chains {
        match chain {
            Chain::Tron => tron_session_properties(&mut properties),
            Chain::Ton => ton_session_properties(&mut properties, accounts),
            Chain::Sui => sui_session_properties(&mut properties, accounts),
            _ => {}
        }
    }
    properties
}

pub fn requires_public_key(chain: Chain) -> bool {
    match chain.chain_type() {
        ChainType::Ton | ChainType::Sui => true,
        ChainType::Ethereum
        | ChainType::Bitcoin
        | ChainType::Solana
        | ChainType::Cosmos
        | ChainType::Tron
        | ChainType::Aptos
        | ChainType::Xrp
        | ChainType::Near
        | ChainType::Stellar
        | ChainType::Algorand
        | ChainType::Polkadot
        | ChainType::Cardano
        | ChainType::HyperCore => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requires_public_key() {
        assert!(requires_public_key(Chain::Ton));
        assert!(requires_public_key(Chain::Sui));
        assert!(!requires_public_key(Chain::Tron));
        assert!(!requires_public_key(Chain::Ethereum));
    }
}
