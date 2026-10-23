use swapper::{SwapperSlippage, config as swap_config};

pub use swap_config::{Config as SwapConfig, SwapQuoteRound, SwapQuoteRoundTiming, get_default_slippage, get_swap_config};

#[uniffi::remote(Record)]
pub struct SwapQuoteRound {
    pub pending_allowed: u32,
    pub answered_minimum: u32,
    pub hold_price_impact_percent: u32,
    pub on_chain: SwapQuoteRoundTiming,
    pub cross_chain: SwapQuoteRoundTiming,
}

#[uniffi::remote(Record)]
pub struct SwapQuoteRoundTiming {
    pub first_quote_wait_ms: u64,
    pub close_ms: u64,
}

#[uniffi::remote(Record)]
pub struct SwapConfig {
    pub default_slippage: SwapperSlippage,
    pub permit2_expiration: u64,
    pub permit2_sig_deadline: u64,
    pub high_price_impact_percent: u32,
    pub quote_round: SwapQuoteRound,
    pub high_slippage_warning_bps: u32,
    pub min_slippage_bps: u32,
    pub max_slippage_bps: u32,
    pub slippage_suggestions_bps: Vec<u32>,
    pub amount_percent_presets: Vec<u32>,
}
