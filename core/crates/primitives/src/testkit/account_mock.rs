use crate::{Account, Chain};

impl Account {
    pub fn mock(chain: Chain, address: &str) -> Self {
        Self {
            chain,
            address: address.to_string(),
            derivation_path: String::new(),
            extended_public_key: None,
        }
    }

    pub fn mock_chains(chains: &[Chain], address: &str) -> Vec<Self> {
        chains.iter().map(|chain| Self::mock(*chain, address)).collect()
    }
}
