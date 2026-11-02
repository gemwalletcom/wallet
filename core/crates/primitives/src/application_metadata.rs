use model_derive::Model;
use serde::{Deserialize, Serialize};

use crate::single_line;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
#[serde(rename_all = "camelCase")]
pub enum ApplicationMetadataSource {
    WalletConnect,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Model)]
#[model(swift = "Equatable, Hashable, Sendable")]
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
        let name = single_line(&self.name);
        for separator in SHORT_NAME_SEPARATORS {
            if let Some(index) = name.find(separator) {
                return name[..index].trim().to_string();
            }
        }
        if let Some((index, _)) = name.char_indices().nth(SHORT_NAME_MAX_LENGTH) {
            return name[..index].to_string();
        }
        name
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

    #[test]
    fn short_name_reads_on_one_line_in_order() {
        let name = |name: &str| {
            ApplicationMetadata {
                name: name.to_string(),
                ..ApplicationMetadata::mock()
            }
            .short_name()
        };

        assert_eq!(name("Uni\u{202e}swap\n\n\nDEX\u{0}"), "Uniswap DEX");
        assert_eq!(name("\u{2066}OpenSea\u{2069} | NFT"), "OpenSea");
    }
}
