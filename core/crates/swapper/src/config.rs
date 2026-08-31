use primitives::Chain;

use crate::{SwapperProvider, SwapperSlippage, SwapperSlippageMode};

pub const DEFAULT_SLIPPAGE_BPS: u32 = 100;
pub const MIN_SLIPPAGE_BPS: u32 = 10;
pub const MAX_SLIPPAGE_BPS: u32 = 2_000;
pub const SLIPPAGE_SUGGESTIONS_BPS: [u32; 3] = [30, 50, 300];
pub const AMOUNT_PERCENT_PRESETS: [u32; 3] = [25, 50, 100];

pub const API_BASE_URL: &str = "https://api.gemwallet.com";

pub fn get_swap_provider_url(provider: SwapperProvider) -> String {
    format!("{API_BASE_URL}/v1/swaps/providers/{}", provider.id())
}

pub fn get_swap_proxy_url(path: &str) -> String {
    format!("{API_BASE_URL}/proxy/swap/{path}")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub high_price_impact_percent: u32,
    pub high_slippage_warning_bps: u32,
    pub min_slippage_bps: u32,
    pub max_slippage_bps: u32,
    pub slippage_suggestions_bps: Vec<u32>,
    pub amount_percent_presets: Vec<u32>,
}

pub fn get_swap_config() -> Config {
    Config {
        default_slippage: SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
        permit2_expiration: 2_592_000, // 30 days
        permit2_sig_deadline: 1800,    // 30 minutes
        high_price_impact_percent: 10,
        high_slippage_warning_bps: 300,
        min_slippage_bps: MIN_SLIPPAGE_BPS,
        max_slippage_bps: MAX_SLIPPAGE_BPS,
        slippage_suggestions_bps: SLIPPAGE_SUGGESTIONS_BPS.to_vec(),
        amount_percent_presets: AMOUNT_PERCENT_PRESETS.to_vec(),
    }
}

pub fn get_default_slippage(chain: &Chain) -> SwapperSlippage {
    match chain {
        Chain::Solana => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS * 3,
            mode: SwapperSlippageMode::Exact,
        },
        _ => SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
    }
}
