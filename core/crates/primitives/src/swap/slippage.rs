use serde::{Deserialize, Serialize};

pub const HUNDRED_PERCENT_IN_BPS: u32 = 10_000;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Slippage {
    pub bps: u32,
    pub mode: SlippageMode,
}

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum SlippageMode {
    Auto,
    Exact,
}

impl From<u32> for Slippage {
    fn from(value: u32) -> Self {
        Slippage {
            bps: value,
            mode: SlippageMode::Exact,
        }
    }
}
