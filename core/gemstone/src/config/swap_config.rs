use primitives::Chain;
use swapper::config as swap_config;

pub use swap_config::{Config as SwapConfig, SlippageConfig as SwapSlippageConfig, get_swap_config};

#[uniffi::remote(Record)]
pub struct SwapConfig {
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub high_price_impact_percent: u32,
    pub slippage: SwapSlippageConfig,
}

#[uniffi::remote(Record)]
pub struct SwapSlippageConfig {
    pub default_bps: u32,
    pub suggestions_bps: Vec<u32>,
    pub min_bps: u32,
    pub max_bps: u32,
    pub high_warning_bps: u32,
}

#[uniffi::export]
pub fn get_default_slippage_bps(chain: &Chain) -> u32 {
    swap_config::get_default_slippage_bps(chain)
}
