use super::rules;
use crate::formatted_number::GemFormattedNumber;
use primitives::{AssetLink, BlockExplorerLink, ChartDateValue, ChartValuePercentage, Currency};

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemChartSection {
    PriceAlerts { count: u32 },
    SetPriceAlert,
    Market { rows: Vec<GemAssetMarketRow> },
    Links { links: Vec<AssetLink> },
}

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemAssetMarketRow {
    MarketCap { value: GemFormattedNumber, rank: Option<i32> },
    FullyDilutedValuation { value: GemFormattedNumber },
    TradingVolume { value: GemFormattedNumber },
    Contract { token_id: String, explorer: Option<BlockExplorerLink> },
    CirculatingSupply { value: GemFormattedNumber },
    TotalSupply { value: GemFormattedNumber },
    MaxSupply { value: GemFormattedNumber },
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
    pub value: GemFormattedNumber,
    pub secondary_value: Option<GemFormattedNumber>,
    pub change: Option<GemFormattedNumber>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemChartData {
    pub value_type: GemChartValueType,
    pub base: f64,
    pub shows_secondary_value: bool,
    pub currency: Currency,
    pub values: Vec<ChartDateValue>,
    pub header: Option<GemChartHeader>,
}

#[uniffi::export]
impl GemChartData {
    pub fn header_at(&self, value: f64) -> GemChartHeader {
        rules::header(self, value, None)
    }
}
