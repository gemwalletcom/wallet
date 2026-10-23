use primitives::Chain;

use crate::{SwapperProvider, SwapperSlippage, SwapperSlippageMode};

pub const DEFAULT_SLIPPAGE_BPS: u32 = 100;
const MIN_SLIPPAGE_BPS: u32 = 10;
const MAX_SLIPPAGE_BPS: u32 = 2_000;
const SLIPPAGE_SUGGESTIONS_BPS: [u32; 3] = [30, 50, 300];
const AMOUNT_PERCENT_PRESETS: [u32; 3] = [25, 50, 100];

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
    pub quote_round: SwapQuoteRound,
    pub high_slippage_warning_bps: u32,
    pub min_slippage_bps: u32,
    pub max_slippage_bps: u32,
    pub slippage_suggestions_bps: Vec<u32>,
    pub amount_percent_presets: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapQuoteRound {
    pub pending_allowed: u32,
    pub answered_minimum: u32,
    pub hold_price_impact_percent: u32,
    pub on_chain: SwapQuoteRoundTiming,
    pub cross_chain: SwapQuoteRoundTiming,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SwapQuoteRoundTiming {
    pub first_quote_wait_ms: u64,
    pub close_ms: u64,
}

impl SwapQuoteRound {
    pub fn timing(&self, from_chain: Chain, to_chain: Chain) -> &SwapQuoteRoundTiming {
        if from_chain == to_chain { &self.on_chain } else { &self.cross_chain }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum QuotePreference {
    Providers { providers: Vec<SwapperProvider>, tolerance_bps: u32 },
    DustFreeMaxAmount { tolerance_bps: u32 },
}

pub fn quote_preferences() -> Vec<QuotePreference> {
    vec![
        QuotePreference::Providers {
            providers: vec![SwapperProvider::Across, SwapperProvider::Thorchain],
            tolerance_bps: 10,
        },
        QuotePreference::Providers {
            providers: vec![SwapperProvider::Orca, SwapperProvider::RaydiumClmm, SwapperProvider::PumpSwap, SwapperProvider::MeteoraDammV2],
            tolerance_bps: 5,
        },
        QuotePreference::Providers {
            providers: vec![SwapperProvider::SunswapV3],
            tolerance_bps: 100,
        },
        QuotePreference::DustFreeMaxAmount { tolerance_bps: 25 },
    ]
}

pub fn get_swap_config() -> Config {
    Config {
        default_slippage: SwapperSlippage {
            bps: DEFAULT_SLIPPAGE_BPS,
            mode: SwapperSlippageMode::Exact,
        },
        permit2_expiration: 2_592_000,
        permit2_sig_deadline: 1800,
        high_price_impact_percent: 10,
        quote_round: SwapQuoteRound {
            pending_allowed: 1,
            answered_minimum: 2,
            hold_price_impact_percent: 1,
            on_chain: SwapQuoteRoundTiming { first_quote_wait_ms: 500, close_ms: 6_000 },
            cross_chain: SwapQuoteRoundTiming {
                first_quote_wait_ms: 1_000,
                close_ms: 12_000,
            },
        },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_quote_round_waits_and_closes_by_whether_the_pair_crosses_networks() {
        let round = get_swap_config().quote_round;

        assert_eq!(round.timing(Chain::Ethereum, Chain::Ethereum), &SwapQuoteRoundTiming { first_quote_wait_ms: 500, close_ms: 6_000 });
        assert_eq!(
            round.timing(Chain::Arbitrum, Chain::Solana),
            &SwapQuoteRoundTiming {
                first_quote_wait_ms: 1_000,
                close_ms: 12_000
            }
        );
    }
}
