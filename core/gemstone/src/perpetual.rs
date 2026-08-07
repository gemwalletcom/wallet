use gem_hypercore::{
    models::websocket::{HyperliquidMethod, HyperliquidRequest, HyperliquidSubscription},
    perpetual_formatter::PerpetualFormatter,
    provider::websocket_mapper::{account_subscriptions, diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data},
};
use primitives::{AutocloseValidation, AutocloseValidator as Validator, PerpetualAccountMode, PerpetualDirection, PerpetualPosition, PerpetualProvider, TpslType};

use crate::models::perpetual::{GemHyperliquidOpenOrder, GemHyperliquidSocketMessage, GemPerpetualSubscription, GemPositionsDiff, GemSubscriptionMethod};

#[derive(Debug, uniffi::Object)]
pub struct Perpetual {
    provider: PerpetualProvider,
}

#[uniffi::export]
impl Perpetual {
    #[uniffi::constructor]
    pub fn new(provider: PerpetualProvider) -> Self {
        Self { provider }
    }

    pub fn minimum_order_usd_amount(&self, price: f64, decimals: i32, leverage: u8) -> u64 {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::minimum_order_usd_amount(price, decimals, leverage),
        }
    }

    pub fn format_price(&self, price: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_price(price, decimals),
        }
    }

    pub fn format_size(&self, size: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_size(size, decimals),
        }
    }
}

#[derive(Debug, uniffi::Object)]
pub struct Hyperliquid {}

impl Default for Hyperliquid {
    fn default() -> Self {
        Self::new()
    }
}

#[uniffi::export]
impl Hyperliquid {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {}
    }

    pub fn account_subscriptions(&self, address: String, mode: PerpetualAccountMode) -> Vec<GemPerpetualSubscription> {
        account_subscriptions(address, mode).into_iter().map(GemPerpetualSubscription::from).collect()
    }

    pub fn parse_websocket_data(&self, data: Vec<u8>, mode: PerpetualAccountMode) -> Result<GemHyperliquidSocketMessage, crate::GemstoneError> {
        Ok(parse_websocket_data(&data, mode)?)
    }

    pub fn websocket_request(&self, method: GemSubscriptionMethod, subscription: GemPerpetualSubscription) -> Result<String, crate::GemstoneError> {
        Ok(serde_json::to_string(&HyperliquidRequest {
            method: method.map(),
            subscription: subscription.map(),
        })?)
    }

    pub fn diff_clearinghouse_positions(&self, new_positions: Vec<PerpetualPosition>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
        diff_clearinghouse_positions(new_positions, existing_positions)
    }

    pub fn diff_open_orders_positions(&self, orders: Vec<GemHyperliquidOpenOrder>, existing_positions: Vec<PerpetualPosition>) -> GemPositionsDiff {
        diff_open_orders_positions(&orders, existing_positions)
    }
}

#[uniffi::remote(Enum)]
pub enum TpslType {
    TakeProfit,
    StopLoss,
}

#[uniffi::remote(Enum)]
pub enum AutocloseValidation {
    Valid,
    InvalidAmount,
    TriggerMustBeHigher,
    TriggerMustBeLower,
}

#[derive(Debug, uniffi::Object)]
pub struct AutocloseValidator {
    inner: Validator,
}

#[uniffi::export]
impl AutocloseValidator {
    #[uniffi::constructor]
    pub fn new(trigger_type: TpslType, direction: PerpetualDirection, market_price: f64) -> Self {
        Self {
            inner: Validator::new(trigger_type, direction, market_price),
        }
    }

    pub fn validate(&self, price: f64) -> AutocloseValidation {
        self.inner.validate(price)
    }
}

impl GemSubscriptionMethod {
    fn map(self) -> HyperliquidMethod {
        match self {
            Self::Subscribe => HyperliquidMethod::Subscribe,
            Self::Unsubscribe => HyperliquidMethod::Unsubscribe,
        }
    }
}

impl From<HyperliquidSubscription> for GemPerpetualSubscription {
    fn from(value: HyperliquidSubscription) -> Self {
        match value {
            HyperliquidSubscription::AccountState { address } => Self::AccountState { address },
            HyperliquidSubscription::SpotState { address } => Self::SpotState { address },
            HyperliquidSubscription::OpenOrders { address } => Self::OpenOrders { address },
            HyperliquidSubscription::Candle { symbol, interval } => Self::Candle { symbol, interval },
            HyperliquidSubscription::MarketData { symbol } => Self::MarketData { symbol },
            HyperliquidSubscription::MarketPrices => Self::MarketPrices,
        }
    }
}

impl GemPerpetualSubscription {
    fn map(self) -> HyperliquidSubscription {
        match self {
            Self::AccountState { address } => HyperliquidSubscription::AccountState { address },
            Self::SpotState { address } => HyperliquidSubscription::SpotState { address },
            Self::OpenOrders { address } => HyperliquidSubscription::OpenOrders { address },
            Self::Candle { symbol, interval } => HyperliquidSubscription::Candle { symbol, interval },
            Self::MarketData { symbol } => HyperliquidSubscription::MarketData { symbol },
            Self::MarketPrices => HyperliquidSubscription::MarketPrices,
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_account_subscriptions() {
        let hyperliquid = Hyperliquid::new();

        assert_eq!(
            hyperliquid.account_subscriptions("0x123".to_string(), PerpetualAccountMode::Standard),
            vec![
                GemPerpetualSubscription::AccountState { address: "0x123".to_string() },
                GemPerpetualSubscription::OpenOrders { address: "0x123".to_string() },
            ]
        );
        assert_eq!(
            hyperliquid.account_subscriptions("0x123".to_string(), PerpetualAccountMode::Unified),
            vec![
                GemPerpetualSubscription::AccountState { address: "0x123".to_string() },
                GemPerpetualSubscription::OpenOrders { address: "0x123".to_string() },
                GemPerpetualSubscription::SpotState { address: "0x123".to_string() },
            ]
        );
    }

    #[test]
    fn test_websocket_request_maps_generic_subscription() {
        let request = HyperliquidRequest {
            method: GemSubscriptionMethod::Subscribe.map(),
            subscription: GemPerpetualSubscription::AccountState { address: "0x123".to_string() }.map(),
        };

        assert_eq!(
            serde_json::to_value(request).unwrap(),
            json!({
                "method": "subscribe",
                "subscription": {
                    "type": "clearinghouseState",
                    "user": "0x123",
                },
            })
        );
    }
}
