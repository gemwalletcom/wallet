mod alien;
mod approval;
mod chainlink;
mod clmm;
pub mod cross_chain;
mod deployment;
mod eth_address;
mod fee_token;
pub mod fees;
mod provider_group;
mod ranking;
pub use ranking::rank_quotes;
mod swapper_trait;

#[cfg(any(test, feature = "testkit"))]
pub mod testkit;

pub mod across;
pub mod bridgers;
pub mod cetus_clmm;
pub mod chainflip;
pub mod client_factory;
pub mod config;
pub mod dedust;
pub mod error;
pub mod hyperion;
pub mod hyperliquid;
pub mod jupiter;
pub mod mayan;
mod meteora_damm_v2;
mod meteora_dlmm;
pub mod models;
pub mod near_intents;
pub mod okx;
mod orca;
pub mod panora;
pub mod permit2_data;
mod pump_swap;
mod raydium_amm;
mod raydium_clmm;
mod raydium_cpmm;
pub mod relay;
mod route_cache;
mod route_path;
mod solana_dex;
pub mod squid;
pub mod stonfi;
pub mod swapper;
pub mod swaps_xyz;
pub mod thorchain;
pub mod uniswap;

use number_formatter::BigNumberFormatter;

pub fn amount_to_value(token: &str, decimals: u32) -> Option<String> {
    let cleaned = token.replace([',', '_'], "");
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.contains('.') { BigNumberFormatter::value_from_amount(&cleaned, decimals).ok() } else { Some(cleaned) }
}

#[cfg(feature = "reqwest_provider")]
pub use alien::reqwest_provider::NativeProvider;
pub use alien::{AlienError, HttpMethod, RpcClient, RpcProvider, Target};
pub use deployment::Deployment;
pub use error::SwapperError;
pub use models::*;
pub use swapper_trait::Swapper;

pub type SwapperProvider = primitives::SwapProvider;
pub type SwapperProviderMode = primitives::swap::SwapProviderMode;
pub type SwapperQuoteAsset = primitives::swap::QuoteAsset;
pub type SwapperSlippage = primitives::swap::Slippage;
pub type SwapperSlippageMode = primitives::swap::SlippageMode;
pub type SwapperQuoteData = primitives::swap::SwapQuoteData;
