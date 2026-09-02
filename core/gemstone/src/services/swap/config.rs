use primitives::Chain;
use primitives::swap::{SwapPriceImpact, SwapQuote};
use swapper::{Quote, SwapperSlippage};

use super::rules;
use crate::config::swap_config::get_default_slippage;
use crate::models::swap::{GemSwapValue, swap_price_impact};

#[derive(Default, uniffi::Object)]
pub struct GemSwapQuoteService {}

#[uniffi::export]
impl GemSwapQuoteService {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn refresh_interval_milliseconds(&self) -> u64 {
        rules::quote_refresh_interval_milliseconds()
    }

    pub fn quote(&self, quote: Quote) -> SwapQuote {
        rules::swap_quote(&quote)
    }

    pub fn price_impact(&self, pay: GemSwapValue, receive: GemSwapValue) -> Option<SwapPriceImpact> {
        swap_price_impact(&pay, &receive)
    }

    pub fn default_slippage(&self, chain: Chain) -> SwapperSlippage {
        get_default_slippage(&chain)
    }
}
