use crate::SwapperProvider;
use primitives::Chain;

pub const DEFAULT_SLIPPAGE_BPS: u32 = 100;

pub const API_BASE_URL: &str = "https://api.gemwallet.com";

pub fn get_swap_provider_url(provider: SwapperProvider) -> String {
    format!("{API_BASE_URL}/v1/swaps/providers/{}", provider.id())
}

pub fn get_swap_proxy_url(path: &str) -> String {
    format!("{API_BASE_URL}/proxy/swap/{path}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub high_price_impact_percent: u32,
    pub slippage: SlippageConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SlippageConfig {
    pub default_bps: u32,
    pub suggestions_bps: Vec<u32>,
    pub min_bps: u32,
    pub max_bps: u32,
    pub high_warning_bps: u32,
}

pub fn get_swap_config() -> Config {
    Config {
        permit2_expiration: 2_592_000, // 30 days
        permit2_sig_deadline: 1800,    // 30 minutes
        high_price_impact_percent: 10,
        slippage: SlippageConfig {
            default_bps: DEFAULT_SLIPPAGE_BPS,
            suggestions_bps: vec![30, 50, 300],
            min_bps: 10,
            max_bps: 2000,
            high_warning_bps: 300,
        },
    }
}

pub fn get_default_slippage_bps(chain: &Chain) -> u32 {
    match chain {
        Chain::Solana => DEFAULT_SLIPPAGE_BPS * 3,
        _ => DEFAULT_SLIPPAGE_BPS,
    }
}
