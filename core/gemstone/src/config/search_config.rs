pub const ASSETS_INITIAL_LIMIT: u32 = 12;
pub const ASSETS_TAG_LIMIT: u32 = 18;
pub const ASSETS_SEARCH_LIMIT: u32 = 25;
pub const PERPETUALS_PREVIEW_LIMIT: u32 = 3;
pub const RESULTS_LIMIT: u32 = 100;

#[derive(uniffi::Record, Clone, Debug, PartialEq, Eq)]
pub struct WalletSearchConfig {
    pub assets_initial_limit: u32,
    pub assets_tag_limit: u32,
    pub assets_search_limit: u32,
    pub perpetuals_preview_limit: u32,
    pub results_limit: u32,
}

pub fn get_wallet_search_config() -> WalletSearchConfig {
    WalletSearchConfig {
        assets_initial_limit: ASSETS_INITIAL_LIMIT,
        assets_tag_limit: ASSETS_TAG_LIMIT,
        assets_search_limit: ASSETS_SEARCH_LIMIT,
        perpetuals_preview_limit: PERPETUALS_PREVIEW_LIMIT,
        results_limit: RESULTS_LIMIT,
    }
}
