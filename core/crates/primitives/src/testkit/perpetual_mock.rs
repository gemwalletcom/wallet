use chrono::{TimeZone, Utc};

use crate::{
    Asset, AssetId, CancelOrderData, Chain, Perpetual, PerpetualConfirmData, PerpetualDirection, PerpetualId, PerpetualMarginType, PerpetualModifyConfirmData,
    PerpetualModifyPositionType, PerpetualOrderType, PerpetualPosition, PerpetualProvider, PerpetualTriggerOrder, TPSLOrderData,
    chart::ChartDateValue,
    portfolio::{PerpetualPortfolio, PerpetualPortfolioTimeframeData},
};

impl PerpetualConfirmData {
    pub fn mock(direction: PerpetualDirection, asset_index: u32, take_profit: Option<String>, stop_loss: Option<String>) -> Self {
        Self {
            direction,
            margin_type: PerpetualMarginType::Cross,
            base_asset: Asset::from_chain(Chain::HyperCore),
            asset_index: asset_index as i32,
            price: "123.45".to_string(),
            fiat_value: 100.0,
            size: "2.5".to_string(),
            slippage: 0.01,
            leverage: 5,
            pnl: None,
            entry_price: None,
            market_price: 123.45,
            margin_amount: 50.0,
            take_profit,
            stop_loss,
        }
    }
}

impl PerpetualPosition {
    pub fn mock() -> Self {
        Self {
            id: "1".to_string(),
            perpetual_id: PerpetualId::new(PerpetualProvider::Hypercore, "BTC"),
            asset_id: AssetId::from_token(Chain::HyperCore, "perpetual::BTC"),
            size: 1.0,
            size_value: 100.0,
            leverage: 5,
            entry_price: 100.0,
            liquidation_price: Some(95.0),
            margin_type: PerpetualMarginType::Cross,
            direction: PerpetualDirection::Long,
            margin_amount: 20.0,
            take_profit: None,
            stop_loss: None,
            pnl: 0.0,
            funding: None,
        }
    }
}

impl Perpetual {
    pub fn mock() -> Self {
        Self {
            id: PerpetualId::new(PerpetualProvider::Hypercore, "BTC"),
            name: "BTC".to_string(),
            provider: PerpetualProvider::Hypercore,
            asset_id: AssetId::from_chain(Chain::HyperCore),
            identifier: "0".to_string(),
            price: 100.0,
            price_percent_change_24h: 0.0,
            open_interest: 0.0,
            volume_24h: 0.0,
            funding: 0.0,
            max_leverage: 20,
            is_isolated_only: true,
        }
    }
}

impl PerpetualTriggerOrder {
    pub fn mock(price: f64) -> Self {
        Self {
            price,
            order_type: PerpetualOrderType::Limit,
            order_id: "order".to_string(),
        }
    }
}

impl PerpetualModifyPositionType {
    pub fn mock_tpsl(take_profit: Option<&str>, stop_loss: Option<&str>) -> Self {
        Self::Tpsl {
            order: TPSLOrderData {
                direction: PerpetualDirection::Long,
                take_profit: take_profit.map(str::to_string),
                stop_loss: stop_loss.map(str::to_string),
                size: "1".to_string(),
            },
        }
    }

    pub fn mock_cancel(order_ids: Vec<u64>) -> Self {
        Self::Cancel {
            orders: order_ids.into_iter().map(|order_id| CancelOrderData { asset_index: 0, order_id }).collect(),
        }
    }
}

impl PerpetualModifyConfirmData {
    pub fn mock(modify_types: Vec<PerpetualModifyPositionType>, take_profit_order_id: Option<u64>, stop_loss_order_id: Option<u64>) -> Self {
        Self {
            base_asset: Asset::mock(),
            asset_index: 0,
            modify_types,
            take_profit_order_id,
            stop_loss_order_id,
        }
    }
}

impl PerpetualPortfolioTimeframeData {
    pub fn mock() -> Self {
        let date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        Self {
            account_value_history: vec![ChartDateValue { date, value: 1000.0 }],
            pnl_history: vec![ChartDateValue { date, value: 50.0 }],
            volume: 5000.0,
        }
    }
}

impl PerpetualPortfolio {
    pub fn mock() -> Self {
        Self {
            day: Some(PerpetualPortfolioTimeframeData::mock()),
            week: None,
            month: None,
            all_time: None,
            account_summary: None,
        }
    }
}
