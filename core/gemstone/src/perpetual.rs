use std::sync::{Mutex, MutexGuard};

use gem_hypercore::{
    models::websocket::{HyperliquidRequest, HyperliquidSubscription},
    perpetual_formatter::PerpetualFormatter,
    provider::{websocket_mapper::account_subscriptions, websocket_subscriptions::WebSocketSubscriptions},
};
use primitives::{
    AutocloseEstimator as Estimator, AutocloseValidation, AutocloseValidator as Validator, PerpetualAccountMode, PerpetualConfirmData, PerpetualDirection, PerpetualProvider,
    PerpetualType, TpslType,
};

use crate::config::perpetual_config::HYPERLIQUID_DEPOSIT_ADDRESS;
use crate::models::custom_types::GemBigInt;
use crate::models::perpetual::GemPerpetualSubscription;
use crate::models::{GemAsset, GemTransactionInputType};
use crate::services::perpetual::model::{GemPerpetualCloseInput, GemPerpetualOrderInput};
use crate::services::perpetual::rules as perpetual_rules;
use crate::services::transfer::model::{GemRecipient, GemTransferData};

const HYPERLIQUID_NAME: &str = "Hyperliquid";

#[derive(Debug, uniffi::Object)]
pub struct GemPerpetual {
    provider: PerpetualProvider,
}

#[uniffi::export]
impl GemPerpetual {
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

    pub fn funding_apr(&self, funding: f64) -> f64 {
        perpetual_rules::funding_apr(funding)
    }

    pub fn recipient(&self) -> GemRecipient {
        GemRecipient {
            address: String::new(),
            name: Some(self.name().to_string()),
            memo: None,
            references: vec![],
        }
    }

    pub fn deposit_recipient(&self) -> GemRecipient {
        let address = match self.provider {
            PerpetualProvider::Hypercore => HYPERLIQUID_DEPOSIT_ADDRESS.to_string(),
        };
        GemRecipient { address, ..self.recipient() }
    }
}

impl GemPerpetual {
    pub fn order(&self, input: GemPerpetualOrderInput) -> PerpetualType {
        perpetual_rules::order(self.provider.clone(), input)
    }
    pub fn close_order(&self, input: GemPerpetualCloseInput) -> PerpetualConfirmData {
        perpetual_rules::close_order(self.provider.clone(), input)
    }
    pub fn transfer_data(&self, asset: GemAsset, perpetual_type: PerpetualType, value: GemBigInt, use_max_amount: bool) -> GemTransferData {
        GemTransferData {
            input_type: GemTransactionInputType::Perpetual { asset, perpetual_type },
            recipient: self.recipient(),
            value,
            use_max_amount,
            minimum_value: None,
        }
    }
}

impl GemPerpetual {
    fn name(&self) -> &'static str {
        match self.provider {
            PerpetualProvider::Hypercore => HYPERLIQUID_NAME,
        }
    }
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

#[derive(Debug, uniffi::Object)]
pub struct GemAutocloseEstimator {
    inner: Estimator,
}

#[uniffi::export]
impl GemAutocloseEstimator {
    #[uniffi::constructor]
    pub fn new(entry_price: f64, position_size: f64, direction: PerpetualDirection, leverage: u8) -> Self {
        Self {
            inner: Estimator::new(entry_price, position_size, direction, leverage),
        }
    }

    #[uniffi::constructor]
    pub fn for_open(market_price: f64, size: f64, leverage: u8, direction: PerpetualDirection) -> Self {
        Self {
            inner: Estimator::for_open(market_price, size, leverage, direction),
        }
    }

    pub fn has_size(&self) -> bool {
        self.inner.has_size()
    }

    pub fn percent_suggestions(&self) -> Vec<u8> {
        crate::config::perpetual_config::get_autoclose_suggestions(self.inner.leverage)
    }

    pub fn pnl(&self, price: f64) -> f64 {
        self.inner.pnl(price)
    }

    pub fn roe(&self, price: f64) -> f64 {
        self.inner.roe(price)
    }

    pub fn target_price_from_roe(&self, roe_percent: i32, trigger_type: TpslType) -> f64 {
        self.inner.target_price_from_roe(roe_percent, trigger_type)
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
    pub(crate) fn map(self) -> HyperliquidSubscription {
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
