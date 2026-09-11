use primitives::{AssetLink, BlockExplorerLink, ChartDateValue, ChartValuePercentage};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartSection {
    PriceAlerts { count: u32 },
    SetPriceAlert,
    Market { rows: Vec<GemAssetMarketRow> },
    Links { links: Vec<AssetLink> },
}

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

#[derive(Debug, Clone, Copy, PartialEq, uniffi::Enum)]
pub enum GemChartValueType {
    Price,
    PriceChange,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartHeader {
    pub value: f64,
    pub secondary_value: Option<f64>,
    pub change_percentage: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartData {
    pub value_type: GemChartValueType,
    pub base: f64,
    pub shows_secondary_value: bool,
    pub values: Vec<ChartDateValue>,
    pub header: Option<GemChartHeader>,
}
