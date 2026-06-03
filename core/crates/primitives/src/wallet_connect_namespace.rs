use crate::{Chain, ChainAddress, ChainType};
use serde::Serialize;
use std::str::FromStr;
use strum::{AsRefStr, EnumString};

#[derive(Debug, Serialize, AsRefStr, EnumString)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum WalletConnectCAIP2 {
    Eip155,
    Solana,
    Cosmos,
    Algorand,
    Sui,
    Ton,
    Tron,
}

impl WalletConnectCAIP2 {
    pub fn get_namespace(chain: Chain) -> Option<String> {
        match chain.chain_type() {
            ChainType::Ethereum => Some(WalletConnectCAIP2::Eip155.as_ref().to_string()),
            ChainType::Solana => Some(WalletConnectCAIP2::Solana.as_ref().to_string()),
            ChainType::Cosmos => Some(format!("{}:{}", WalletConnectCAIP2::Cosmos.as_ref(), chain.network_id())),
            ChainType::Algorand => Some(WalletConnectCAIP2::Algorand.as_ref().to_string()),
            ChainType::Sui => Some(WalletConnectCAIP2::Sui.as_ref().to_string()),
            ChainType::Ton => Some(WalletConnectCAIP2::Ton.as_ref().to_string()),
            ChainType::Tron => Some(WalletConnectCAIP2::Tron.as_ref().to_string()),
            ChainType::Bitcoin | ChainType::Aptos | ChainType::Xrp | ChainType::Near | ChainType::Stellar | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => None,
        }
    }

    pub fn get_chain_type(namespace: String) -> Option<ChainType> {
        match WalletConnectCAIP2::from_str(&namespace).ok()? {
            WalletConnectCAIP2::Eip155 => Some(ChainType::Ethereum),
            WalletConnectCAIP2::Solana => Some(ChainType::Solana),
            WalletConnectCAIP2::Cosmos => Some(ChainType::Cosmos),
            WalletConnectCAIP2::Algorand => Some(ChainType::Algorand),
            WalletConnectCAIP2::Sui => Some(ChainType::Sui),
            WalletConnectCAIP2::Ton => Some(ChainType::Ton),
            WalletConnectCAIP2::Tron => Some(ChainType::Tron),
        }
    }

    pub fn get_chain(namespace: String, reference: String) -> Option<Chain> {
        let namespace = WalletConnectCAIP2::from_str(&namespace).ok()?;
        match namespace {
            WalletConnectCAIP2::Eip155 | WalletConnectCAIP2::Cosmos => {
                let chain_type = Self::get_chain_type(namespace.as_ref().to_string())?;
                Chain::all()
                    .into_iter()
                    .filter(|chain| chain.chain_type() == chain_type && chain.network_id() == reference)
                    .collect::<Vec<_>>()
                    .first()
                    .cloned()
            }
            WalletConnectCAIP2::Solana => Some(Chain::Solana),
            WalletConnectCAIP2::Algorand => Some(Chain::Algorand),
            WalletConnectCAIP2::Sui => Some(Chain::Sui),
            WalletConnectCAIP2::Ton => Some(Chain::Ton),
            WalletConnectCAIP2::Tron => Some(Chain::Tron),
        }
    }

    pub fn get_reference(chain: Chain) -> Option<String> {
        match chain.chain_type() {
            ChainType::Ethereum => Some(chain.network_id().to_string()),
            ChainType::Solana => Some(chain.network_id().chars().take(32).collect()),
            ChainType::Cosmos => Self::get_namespace(chain).map(|namespace| format!("{}:{}", namespace, chain.network_id())),
            ChainType::Algorand => Some("wGHE2Pwdvd7S12BL5FaOP20EGYesN73k".to_string()),
            ChainType::Sui => Some("mainnet".to_string()),
            ChainType::Ton => Some("-239".to_string()),
            ChainType::Tron => Some(chain.network_id().to_string()),
            ChainType::Bitcoin | ChainType::Aptos | ChainType::Xrp | ChainType::Near | ChainType::Stellar | ChainType::Polkadot | ChainType::Cardano | ChainType::HyperCore => None,
        }
    }

    pub fn parse_chain_id(chain_id: String) -> Option<Chain> {
        let (namespace, reference) = Self::parse_chain_id_parts(&chain_id)?;
        Self::get_chain(namespace.to_string(), reference.to_string())
    }

    pub fn parse_account(account: String) -> Option<ChainAddress> {
        let mut parts = account.split(':');
        let namespace = parts.next()?;
        let reference = parts.next()?;
        let address = parts.next()?;
        if parts.next().is_some() || address.is_empty() {
            return None;
        }

        Some(ChainAddress::new(Self::get_chain(namespace.to_string(), reference.to_string())?, address.to_string()))
    }

    pub fn resolve_chain(chain_id: Option<String>) -> Result<Chain, String> {
        let chain_id = chain_id.ok_or("Chain ID is required")?;
        if Self::parse_chain_id_parts(&chain_id).is_none() {
            return Err("Invalid chain ID format".to_string());
        }
        Self::parse_chain_id(chain_id).ok_or("Unsupported chain".to_string())
    }

    fn parse_chain_id_parts(chain_id: &str) -> Option<(&str, &str)> {
        let mut parts = chain_id.split(':');
        let namespace = parts.next()?;
        let reference = parts.next()?;
        if parts.next().is_some() {
            return None;
        }
        Some((namespace, reference))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_chain_type() {
        assert_eq!(WalletConnectCAIP2::get_chain_type("eip155".to_string()), Some(ChainType::Ethereum));
        assert_eq!(WalletConnectCAIP2::get_chain_type("solana".to_string()), Some(ChainType::Solana));
        assert_eq!(WalletConnectCAIP2::get_chain_type("cosmos".to_string()), Some(ChainType::Cosmos));
        assert_eq!(WalletConnectCAIP2::get_chain_type("algorand".to_string()), Some(ChainType::Algorand));
        assert_eq!(WalletConnectCAIP2::get_chain_type("sui".to_string()), Some(ChainType::Sui));
        assert_eq!(WalletConnectCAIP2::get_chain_type("ton".to_string()), Some(ChainType::Ton));
        assert_eq!(WalletConnectCAIP2::get_chain_type("tron".to_string()), Some(ChainType::Tron));
        assert_eq!(WalletConnectCAIP2::get_chain_type("bip122".to_string()), None);
        assert_eq!(WalletConnectCAIP2::get_chain_type("unknown".to_string()), None);
    }

    #[test]
    fn test_get_chain() {
        assert_eq!(WalletConnectCAIP2::get_chain("eip155".to_string(), "1".to_string()), Some(Chain::Ethereum));
        assert_eq!(WalletConnectCAIP2::get_chain("eip155".to_string(), "56".to_string()), Some(Chain::SmartChain));
        assert_eq!(WalletConnectCAIP2::get_chain("solana".to_string(), "ignored".to_string()), Some(Chain::Solana));
        assert_eq!(WalletConnectCAIP2::get_chain("sui".to_string(), "mainnet".to_string()), Some(Chain::Sui));
        assert_eq!(WalletConnectCAIP2::get_chain("ton".to_string(), "-239".to_string()), Some(Chain::Ton));
        assert_eq!(WalletConnectCAIP2::get_chain("tron".to_string(), "0x2b6653dc".to_string()), Some(Chain::Tron));
        assert_eq!(WalletConnectCAIP2::get_chain("bip122".to_string(), "000000000019d6689c085ae165831e93".to_string()), None);
    }

    #[test]
    fn test_resolve_chain() {
        assert_eq!(WalletConnectCAIP2::resolve_chain(Some("eip155:1".to_string())), Ok(Chain::Ethereum));
        assert_eq!(
            WalletConnectCAIP2::resolve_chain(Some("solana:5eykt4UsFv8P8NJdTREpY1vzqKqZKvdp".to_string())),
            Ok(Chain::Solana)
        );
        assert_eq!(WalletConnectCAIP2::resolve_chain(Some("sui:mainnet".to_string())), Ok(Chain::Sui));
        assert_eq!(WalletConnectCAIP2::resolve_chain(Some("ton:-239".to_string())), Ok(Chain::Ton));
        assert_eq!(WalletConnectCAIP2::resolve_chain(Some("tron:0x2b6653dc".to_string())), Ok(Chain::Tron));
        assert!(WalletConnectCAIP2::resolve_chain(Some("bip122:000000000019d6689c085ae165831e93".to_string())).is_err());
        assert!(WalletConnectCAIP2::resolve_chain(Some("invalid".to_string())).is_err());
        assert!(WalletConnectCAIP2::resolve_chain(Some("eip155:1:extra".to_string())).is_err());
        assert!(WalletConnectCAIP2::resolve_chain(None).is_err());
        assert!(WalletConnectCAIP2::resolve_chain(Some("unknown:chain".to_string())).is_err());
    }

    #[test]
    fn test_parse_account() {
        assert_eq!(
            WalletConnectCAIP2::parse_account("eip155:8453:0x0000000000000000000000000000000000000001".to_string()),
            Some(ChainAddress::new(Chain::Base, "0x0000000000000000000000000000000000000001".to_string()))
        );
        assert_eq!(WalletConnectCAIP2::parse_account("eip155:8453".to_string()), None);
        assert_eq!(WalletConnectCAIP2::parse_account("eip155:8453:".to_string()), None);
        assert_eq!(WalletConnectCAIP2::parse_account("eip155:8453:0x1:extra".to_string()), None);
        assert_eq!(WalletConnectCAIP2::parse_account("eip155:99999:0x1".to_string()), None);
    }
}
