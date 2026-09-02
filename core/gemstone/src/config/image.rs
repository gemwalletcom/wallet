use primitives::{AssetId, Chain, ImageFormatter};

use super::public::ASSETS_URL;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemImage {
    Asset { asset_id: AssetId },
    Validator { chain: Chain, validator_id: String },
    NftAsset { asset_id: String },
    AssetList { list_id: String },
}

#[uniffi::export]
impl GemImage {
    pub fn url(&self) -> String {
        match self {
            Self::Asset { asset_id } => ImageFormatter::get_asset_url_for_asset_id(ASSETS_URL, asset_id.clone()),
            Self::Validator { chain, validator_id } => ImageFormatter::get_validator_url(ASSETS_URL, chain.as_ref(), validator_id),
            Self::NftAsset { asset_id } => ImageFormatter::get_nft_asset_url(&format!("{ASSETS_URL}/nft"), asset_id),
            Self::AssetList { list_id } => ImageFormatter::get_asset_list_url(ASSETS_URL, list_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_urls_share_the_assets_host() {
        assert_eq!(
            GemImage::Asset {
                asset_id: AssetId::from(Chain::Ethereum, Some("0xdac".to_string()))
            }
            .url(),
            "https://assets.gemwallet.com/blockchains/ethereum/assets/0xdac/logo.png"
        );
        assert_eq!(
            GemImage::Validator {
                chain: Chain::Cosmos,
                validator_id: "cosmosvaloper1".to_string()
            }
            .url(),
            "https://assets.gemwallet.com/blockchains/cosmos/validators/cosmosvaloper1/logo.png"
        );
        assert_eq!(
            GemImage::NftAsset {
                asset_id: "ethereum_0xabc::1".to_string()
            }
            .url(),
            "https://assets.gemwallet.com/nft/assets/ethereum_0xabc::1/preview"
        );
        assert_eq!(
            GemImage::AssetList { list_id: "trending".to_string() }.url(),
            "https://assets.gemwallet.com/lists/trending.png"
        );
    }
}
