use std::sync::{Mutex, MutexGuard};

use gem_hypercore::{
    models::websocket::{HyperliquidRequest, HyperliquidSubscription},
    perpetual_formatter::PerpetualFormatter,
    provider::{
        websocket_mapper::{account_subscriptions, diff_clearinghouse_positions, diff_open_orders_positions, parse_websocket_data},
        websocket_subscriptions::WebSocketSubscriptions,
    },
};
use primitives::{AutocloseValidation, AutocloseValidator as Validator, PerpetualAccountMode, PerpetualDirection, PerpetualPosition, PerpetualProvider, TpslType};

use crate::models::perpetual::{GemHyperliquidOpenOrder, GemHyperliquidSocketMessage, GemPerpetualSubscription, GemPositionsDiff};

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

    pub fn parse_websocket_data(&self, data: Vec<u8>, mode: PerpetualAccountMode) -> Result<GemHyperliquidSocketMessage, crate::GemstoneError> {
        Ok(parse_websocket_data(&data, mode)?)
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

#[derive(Debug, Default, uniffi::Object)]
pub struct HyperliquidSubscriptions {
    state: Mutex<WebSocketSubscriptions>,
}

#[uniffi::export]
impl HyperliquidSubscriptions {
    #[uniffi::constructor]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(WebSocketSubscriptions::new()),
        }
    }

    pub fn subscribe(&self, subscription: GemPerpetualSubscription) -> Result<Vec<String>, crate::GemstoneError> {
        encode(self.state().subscribe(subscription.map()))
    }

    pub fn unsubscribe(&self, subscription: GemPerpetualSubscription) -> Result<Vec<String>, crate::GemstoneError> {
        encode(self.state().unsubscribe(&subscription.map()))
    }

    pub fn connected(&self, address: String, mode: PerpetualAccountMode) -> Result<Vec<String>, crate::GemstoneError> {
        encode(self.state().connected(account_subscriptions(address, mode)))
    }

    pub fn disconnected(&self) {
        self.state().disconnected();
    }
}

impl HyperliquidSubscriptions {
    fn state(&self) -> MutexGuard<'_, WebSocketSubscriptions> {
        match self.state.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }
}

fn encode(requests: Vec<HyperliquidRequest>) -> Result<Vec<String>, crate::GemstoneError> {
    Ok(requests.iter().map(serde_json::to_string).collect::<Result<Vec<_>, _>>()?)
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
    fn test_connected_subscribes_account_subscriptions() {
        let subscriptions = HyperliquidSubscriptions::new();

        let requests = subscriptions.connected("0x123".to_string(), PerpetualAccountMode::Unified).unwrap();

        assert_eq!(
            requests,
            vec![
                r#"{"method":"subscribe","subscription":{"type":"clearinghouseState","user":"0x123"}}"#,
                r#"{"method":"subscribe","subscription":{"type":"openOrders","user":"0x123"}}"#,
                r#"{"method":"subscribe","subscription":{"type":"spotState","user":"0x123"}}"#,
            ]
        );
    }

    #[test]
    fn test_subscriptions_encode_generic_subscription() {
        let subscriptions = HyperliquidSubscriptions::new();
        subscriptions.connected("0x456".to_string(), PerpetualAccountMode::Standard).unwrap();

        let requests = subscriptions.subscribe(GemPerpetualSubscription::AccountState { address: "0x123".to_string() }).unwrap();

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&requests[0]).unwrap(),
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
