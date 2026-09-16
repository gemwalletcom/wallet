use serde::{Deserialize, Serialize};
use typeshare::typeshare;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[typeshare(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum ApplicationMetadataSource {
    WalletConnect,
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

    #[test]
    fn short_name_strips_separators() {
        assert_eq!(
            ApplicationMetadata {
                name: "Polymarket - Buy & Sell".to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "Polymarket"
        );
        assert_eq!(
            ApplicationMetadata {
                name: "Uniswap: Trade Crypto".to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "Uniswap"
        );
        assert_eq!(
            ApplicationMetadata {
                name: "OpenSea | NFT Marketplace".to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "OpenSea"
        );
        assert_eq!(
            ApplicationMetadata {
                name: "  Compound  ".to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "Compound"
        );
        assert_eq!(
            ApplicationMetadata {
                name: "Sushiswap".to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "Sushiswap"
        );
        assert_eq!(
            ApplicationMetadata {
                name: "A".repeat(100),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "A".repeat(80)
        );
        assert_eq!(
            ApplicationMetadata {
                name: "é".repeat(100),
                ..ApplicationMetadata::mock()
            }
            .short_name(),
            "é".repeat(80)
        );
    }
}
