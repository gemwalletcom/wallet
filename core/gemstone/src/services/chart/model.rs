use primitives::{BlockExplorerLink, ChartValuePercentage};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetMarketRow {
    MarketCap { value: f64, rank: Option<i32> },
    FullyDilutedValuation { value: f64 },
    TradingVolume { value: f64 },
    Contract { token_id: String, explorer: Option<BlockExplorerLink> },
    CirculatingSupply { value: f64 },
    TotalSupply { value: f64 },
    MaxSupply { value: f64 },
    AllTimeHigh { value: ChartValuePercentage },
    AllTimeLow { value: ChartValuePercentage },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemAssetMarketRows {
    pub market: Vec<GemAssetMarketRow>,
    pub contract: Vec<GemAssetMarketRow>,
    pub supply: Vec<GemAssetMarketRow>,
    pub all_time: Vec<GemAssetMarketRow>,
}
