use chrono::DateTime;

use crate::{AssetMarket, ChartValuePercentage};

impl AssetMarket {
    pub fn mock() -> Self {
        AssetMarket {
            market_cap: Some(100.0),
            market_cap_fdv: Some(120.0),
            market_cap_rank: Some(1),
            total_volume: Some(10.0),
            circulating_supply: Some(50.0),
            total_supply: Some(60.0),
            max_supply: Some(21.0),
            all_time_high_value: Some(ChartValuePercentage::mock()),
            all_time_low_value: Some(ChartValuePercentage::mock_low()),
            ..AssetMarket::default()
        }
    }

    pub fn mock_partial() -> Self {
        AssetMarket {
            market_cap: None,
            total_volume: None,
            total_supply: None,
            all_time_low_value: None,
            ..AssetMarket::mock()
        }
    }

    pub fn mock_with_rank(rank: i32) -> Self {
        AssetMarket {
            market_cap_rank: Some(rank),
            ..AssetMarket::mock()
        }
    }
}

impl ChartValuePercentage {
    pub fn mock() -> Self {
        ChartValuePercentage {
            date: DateTime::from_timestamp(10, 0).unwrap(),
            value: 1.1,
            percentage: -5.0,
        }
    }

    pub fn mock_low() -> Self {
        ChartValuePercentage {
            date: DateTime::from_timestamp(5, 0).unwrap(),
            value: 0.9,
            percentage: 25.0,
        }
    }
}
