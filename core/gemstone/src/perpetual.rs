use gem_hypercore::{models::websocket::HyperliquidSubscription, perpetual_formatter::PerpetualFormatter};
use primitives::contract_constants::HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS;
use primitives::known_assets::ARBITRUM_USDC;
use primitives::{Asset, AutocloseEstimator as Estimator, AutocloseValidation, AutocloseValidator as Validator, PerpetualConfirmData, PerpetualDirection, PerpetualProvider, PerpetualType, TpslType};

use crate::models::GemAsset;
use crate::models::custom_types::GemBigInt;
use crate::models::perpetual::GemPerpetualSubscription;
use crate::services::perpetual::model::{GemPerpetualCloseInput, GemPerpetualOrderInput};
use crate::services::perpetual::rules as perpetual_rules;
use crate::services::transfer::model::{GemRecipient, GemTransferData};
use primitives::TransactionInputType;

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

    pub fn format_price(&self, price: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_price(price, decimals),
        }
    }

    pub fn format_input_price(&self, price: f64, decimals: i32, decimal_separator: String) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_input_price(price, decimals, decimal_separator.chars().next().unwrap_or('.')),
        }
    }

    pub fn deposit_asset(&self) -> Asset {
        match self.provider {
            PerpetualProvider::Hypercore => ARBITRUM_USDC.clone(),
        }
    }
}

impl GemPerpetual {
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
            PerpetualProvider::Hypercore => HYPERLIQUID_ARBITRUM_DEPOSIT_ADDRESS.to_string(),
        };
        GemRecipient { address, ..self.recipient() }
    }
}

impl GemPerpetual {
    pub fn format_size(&self, size: f64, decimals: i32) -> String {
        match self.provider {
            PerpetualProvider::Hypercore => PerpetualFormatter::format_size(size, decimals),
        }
    }

    pub fn order(&self, input: GemPerpetualOrderInput) -> PerpetualType {
        perpetual_rules::order(self.provider.clone(), input)
    }
    pub fn close_order(&self, input: GemPerpetualCloseInput) -> PerpetualConfirmData {
        perpetual_rules::close_order(self.provider.clone(), input)
    }
    pub fn transfer_data(&self, asset: GemAsset, perpetual_type: PerpetualType, value: GemBigInt, use_max_amount: bool) -> GemTransferData {
        GemTransferData {
            input_type: TransactionInputType::Perpetual { asset, perpetual_type },
            recipient: self.recipient(),
            value,
            use_max_amount,
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

    pub fn validate(&self, price: Option<f64>) -> AutocloseValidation {
        price.map_or(AutocloseValidation::Valid, |price| self.inner.validate(price))
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

    pub fn percent_suggestions(&self) -> Vec<u8> {
        crate::config::perpetual_config::get_autoclose_suggestions(self.inner.leverage)
    }

    pub fn pnl(&self, price: f64) -> f64 {
        self.inner.pnl(price)
    }

    pub fn is_profit(&self, price: Option<f64>, tpsl_type: TpslType) -> bool {
        match price {
            Some(price) => self.inner.pnl(price) >= 0.0,
            None => tpsl_type == TpslType::TakeProfit,
        }
    }

    pub fn target_price_from_roe(&self, roe_percent: i32, trigger_type: TpslType) -> f64 {
        self.inner.target_price_from_roe(roe_percent, trigger_type)
    }
}

impl GemAutocloseEstimator {
    pub fn roe(&self, price: f64) -> f64 {
        self.inner.roe(price)
    }

    pub fn has_size(&self) -> bool {
        self.inner.has_size()
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
    use super::*;

    #[test]
    fn test_a_target_price_reads_as_profit_from_its_pnl_and_falls_back_to_the_trigger_kind() {
        let estimator = GemAutocloseEstimator::new(100.0, 1.0, PerpetualDirection::Long, 5);

        assert!(estimator.is_profit(Some(110.0), TpslType::StopLoss));
        assert!(!estimator.is_profit(Some(90.0), TpslType::TakeProfit));
        assert!(estimator.is_profit(None, TpslType::TakeProfit));
        assert!(!estimator.is_profit(None, TpslType::StopLoss));
    }

    #[test]
    fn test_autoclose_validator_treats_an_unset_price_as_valid() {
        let validator = AutocloseValidator::new(TpslType::TakeProfit, PerpetualDirection::Long, 100.0);

        assert_eq!(validator.validate(None), AutocloseValidation::Valid);
        assert_eq!(validator.validate(Some(90.0)), AutocloseValidation::TriggerMustBeHigher);
    }
}
