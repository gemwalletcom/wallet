use crate::{ContractAddress, Platform, PlatformCoin};

impl ContractAddress {
    pub fn mock_with_address(contract_address: &str) -> Self {
        Self {
            contract_address: contract_address.to_string(),
            platform: Platform {
                name: "Ethereum".to_string(),
                coin: PlatformCoin { slug: "ethereum".to_string() },
            },
        }
    }
}
