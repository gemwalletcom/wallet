use primitives::ImageFormatter as Formatter;

use crate::config::public::ASSETS_URL;

#[derive(Debug, uniffi::Object)]
pub struct ImageFormatter {}

impl Default for ImageFormatter {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl ImageFormatter {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_asset_url(&self, chain: String, token_id: Option<String>) -> String {
        Formatter::get_asset_url(ASSETS_URL, &chain, token_id.as_deref())
    }

    pub fn get_validator_url(&self, chain: String, id: String) -> String {
        Formatter::get_validator_url(ASSETS_URL, &chain, &id)
    }

    pub fn get_nft_asset_url(&self, id: String) -> String {
        Formatter::get_nft_asset_url(&format!("{ASSETS_URL}/nft"), &id)
    }

    pub fn get_list_url(&self, id: String) -> String {
        Formatter::get_list_url(ASSETS_URL, &id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urls() {
        let formatter = ImageFormatter::new();

        assert_eq!(
            formatter.get_asset_url("ethereum".into(), None),
            "https://assets.gemwallet.com/blockchains/ethereum/logo.png"
        );
        assert_eq!(
            formatter.get_asset_url("ethereum".into(), Some("0x1".into())),
            "https://assets.gemwallet.com/blockchains/ethereum/assets/0x1/logo.png"
        );
        assert_eq!(
            formatter.get_validator_url("ethereum".into(), "0x2".into()),
            "https://assets.gemwallet.com/blockchains/ethereum/validators/0x2/logo.png"
        );
        assert_eq!(
            formatter.get_nft_asset_url("ethereum_0xabc::1".into()),
            "https://assets.gemwallet.com/nft/assets/ethereum_0xabc::1/preview"
        );
        assert_eq!(formatter.get_list_url("trending".into()), "https://assets.gemwallet.com/lists/trending.png");
    }
}
