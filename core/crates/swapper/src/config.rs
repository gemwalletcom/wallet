use crate::{SwapperProvider, SwapperSlippage, SwapperSlippageMode};
use primitives::Chain;

pub const DEFAULT_SLIPPAGE_BPS: u32 = 100;
pub const DEFAULT_SWAP_FEE_BPS: u32 = 50;
pub const DEFAULT_CHAINFLIP_FEE_BPS: u32 = 45;

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
