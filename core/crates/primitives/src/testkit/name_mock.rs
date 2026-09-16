use crate::{Chain, NameProvider, NameRecord};

impl NameRecord {
    pub fn mock(name: &str, address: &str) -> Self {
        Self {
            name: name.to_string(),
            chain: Chain::Ethereum,
            address: address.to_string(),
            provider: NameProvider::Ens,
        }
    }
}
