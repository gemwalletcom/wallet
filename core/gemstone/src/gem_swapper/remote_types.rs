use crate::models::custom_types::GemBigUint;

use primitives::{AssetId, Chain};
pub use swapper::{
    AssetList as SwapperAssetList, FetchQuoteData, Options as SwapperOptions, ProviderData as SwapperProviderData, ProviderType as SwapperProviderType, Quote as SwapperQuote,
    QuoteRequest as SwapperQuoteRequest, Route as SwapperRoute, SwapperProvider, SwapperProviderMode, SwapperQuoteAsset, SwapperSlippage, SwapperSlippageMode,
    permit2_data::Permit2Data,
};

pub use crate::models::swap::GemSwapQuoteData;

#[uniffi::remote(Enum)]
pub enum FetchQuoteData {
    Permit2(Permit2Data),
    EstimateGas,
    None,
}

#[uniffi::remote(Record)]
pub struct SwapperAssetList {
    pub chains: Vec<Chain>,
    pub asset_ids: Vec<AssetId>,
}

#[uniffi::remote(Record)]
pub struct SwapperProviderType {
    pub id: SwapperProvider,
    pub name: String,
    pub protocol: String,
    pub protocol_id: String,
    pub mode: SwapperProviderMode,
    pub slippage_mode: SwapperSlippageMode,
}

#[uniffi::remote(Record)]
pub struct SwapperOptions {
    pub slippage: SwapperSlippage,
    pub use_max_amount: bool,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteRequest {
    pub from_asset: SwapperQuoteAsset,
    pub to_asset: SwapperQuoteAsset,
    pub wallet_address: String,
    pub destination_address: String,
    pub value: GemBigUint,
    pub options: SwapperOptions,
}

#[uniffi::remote(Record)]
pub struct SwapperRoute {
    pub input: AssetId,
    pub output: AssetId,
    pub route_data: String,
}

#[uniffi::remote(Record)]
pub struct SwapperProviderData {
    pub provider: SwapperProviderType,
    pub slippage_bps: u32,
    pub routes: Vec<SwapperRoute>,
}

#[uniffi::remote(Record)]
pub struct SwapperQuote {
    pub from_value: GemBigUint,
    pub min_from_value: Option<GemBigUint>,
    pub to_value: GemBigUint,
    pub data: SwapperProviderData,
    pub request: SwapperQuoteRequest,
    pub eta_in_seconds: Option<u32>,
}

#[uniffi::remote(Enum)]
pub enum SwapperProviderMode {
    OnChain,
    CrossChain,
    Bridge,
    OmniChain(Vec<Chain>),
}

#[uniffi::remote(Record)]
pub struct SwapperSlippage {
    pub bps: u32,
    pub mode: SwapperSlippageMode,
}

#[uniffi::remote(Enum)]
pub enum SwapperSlippageMode {
    Auto,
    Exact,
}

#[uniffi::remote(Record)]
pub struct SwapperQuoteAsset {
    pub id: String,
    pub symbol: String,
    pub decimals: u32,
}
