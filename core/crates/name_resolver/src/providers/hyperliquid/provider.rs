use std::error::Error;
use std::str::FromStr;

use alloy_primitives::Address;
use async_trait::async_trait;
use gem_client::ReqwestClient;
use primitives::{Chain, EVMChain, NameProvider};

use super::client::HyperliquidClient;
use super::model::Record;
use crate::model::NameQuery;
use crate::resolver::NameResolver;

pub struct HyperliquidProvider {
    client: HyperliquidClient,
}

impl HyperliquidProvider {
    pub fn new(client: ReqwestClient) -> Self {
        Self { client: HyperliquidClient::new(client) }
    }

    fn is_valid_name(name: &str) -> bool {
        !name.is_empty() && name.split('.').all(|label| !label.is_empty())
    }

    fn map_address(record: Record, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let address = match chain {
            Chain::Hyperliquid => Some(record.name.resolved),
            _ => record.data.chain_addresses.get(&chain.as_slip44().to_string()).cloned(),
        };
        let Some(address) = address else {
            return Ok(None);
        };
        match EVMChain::from_chain(chain) {
            Some(_) => Ok(Some(Address::from_str(&address)?.to_string())),
            None => Ok(Some(address)),
        }
    }
}

#[async_trait]
impl NameResolver for HyperliquidProvider {
    fn provider(&self) -> NameProvider {
        NameProvider::Hyperliquid
    }

    fn domains(&self) -> Vec<&'static str> {
        vec!["hl"]
    }

    fn chains(&self) -> Vec<Chain> {
        vec![Chain::Bitcoin, Chain::Ethereum, Chain::Solana, Chain::Hyperliquid]
    }

    async fn resolve(&self, query: &NameQuery, chain: Chain) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let Some(name) = query.ascii_domain().ok().filter(|name| Self::is_valid_name(name)) else {
            return Ok(None);
        };
        match self.client.get_record(&name).await? {
            Some(record) => Self::map_address(record, chain),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use primitives::Chain;

    use super::HyperliquidProvider;
    use crate::providers::hyperliquid::model::Record;

    #[test]
    fn test_is_valid_name() {
        assert!(HyperliquidProvider::is_valid_name("test.hl"));
        assert!(HyperliquidProvider::is_valid_name("a.b.test.hl"));
        assert!(HyperliquidProvider::is_valid_name("foo-bar.hl"));
        assert!(HyperliquidProvider::is_valid_name("🐈🐈🐈🐈🐈🐈🐈.hl"));

        assert!(!HyperliquidProvider::is_valid_name("test..hl"));
        assert!(!HyperliquidProvider::is_valid_name("test.hl."));
        assert!(!HyperliquidProvider::is_valid_name(".test.hl"));
        assert!(!HyperliquidProvider::is_valid_name(""));
    }

    #[test]
    fn test_map_address() {
        assert_eq!(HyperliquidProvider::map_address(Record::mock(), Chain::Hyperliquid).unwrap().as_deref(), Some("0xF26F5551E96aE5162509B25925fFfa7F07B2D652"));
        assert_eq!(HyperliquidProvider::map_address(Record::mock(), Chain::Ethereum).unwrap().as_deref(), Some("0xb43f5153B1c867BF78ACB3C35aa9b8ae366415c5"));
        assert_eq!(HyperliquidProvider::map_address(Record::mock(), Chain::Solana).unwrap().as_deref(), Some("CKAvaYmwqCbg8nZCUCNj6Cvr11HauALtNoGT7WirPoAp"));
        assert_eq!(HyperliquidProvider::map_address(Record::mock(), Chain::Bitcoin).unwrap(), None);
    }
}
