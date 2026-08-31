use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum ApplicationMetadataSource {
    WalletConnect,
    Payment,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub struct ApplicationMetadata {
    pub name: String,
    pub description: String,
    pub url: String,
    pub icon: String,
    pub source: ApplicationMetadataSource,
}

const SHORT_NAME_SEPARATORS: [char; 3] = ['-', ':', '|'];
const SHORT_NAME_MAX_LENGTH: usize = 80;

impl ApplicationMetadata {
    pub fn short_name(&self) -> String {
        let name = self.name.trim();
        for separator in SHORT_NAME_SEPARATORS {
            if let Some(index) = name.find(separator) {
                return name[..index].trim().to_string();
            }
        }
        if let Some((index, _)) = name.char_indices().nth(SHORT_NAME_MAX_LENGTH) {
            return name[..index].to_string();
        }
        name.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata(name: &str) -> ApplicationMetadata {
        ApplicationMetadata {
            name: name.to_string(),
            description: String::new(),
            url: String::new(),
            icon: String::new(),
            source: ApplicationMetadataSource::WalletConnect,
        }
    }

    #[test]
    fn short_name_strips_separators() {
        assert_eq!(metadata("Polymarket - Buy & Sell").short_name(), "Polymarket");
        assert_eq!(metadata("Uniswap: Trade Crypto").short_name(), "Uniswap");
        assert_eq!(metadata("OpenSea | NFT Marketplace").short_name(), "OpenSea");
        assert_eq!(metadata("  Compound  ").short_name(), "Compound");
        assert_eq!(metadata("Sushiswap").short_name(), "Sushiswap");
        assert_eq!(metadata(&"A".repeat(100)).short_name(), "A".repeat(80));
        assert_eq!(metadata(&"é".repeat(100)).short_name(), "é".repeat(80));
    }
}
