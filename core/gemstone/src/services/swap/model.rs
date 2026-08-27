use primitives::swap::{SwapQuote, SwapQuoteData};

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemSwapTransfer {
    pub quote: SwapQuote,
    pub data: SwapQuoteData,
    pub recipient: String,
    pub value: String,
    pub use_max_amount: bool,
}
