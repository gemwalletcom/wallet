use crate::{PerpetualDirection, TpslType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutocloseValidation {
    Valid,
    InvalidAmount,
    TriggerMustBeHigher,
    TriggerMustBeLower,
}

#[derive(Debug)]
pub struct AutocloseValidator {
    trigger_type: TpslType,
    direction: PerpetualDirection,
    market_price: f64,
}

impl AutocloseValidator {
    pub fn new(trigger_type: TpslType, direction: PerpetualDirection, market_price: f64) -> Self {
        Self {
            trigger_type,
            direction,
            market_price,
        }
    }

    pub fn validate(&self, price: f64) -> AutocloseValidation {
        if price <= 0.0 {
            return AutocloseValidation::InvalidAmount;
        }
        let must_be_above = match self.trigger_type {
            TpslType::TakeProfit => self.direction == PerpetualDirection::Long,
            TpslType::StopLoss => self.direction == PerpetualDirection::Short,
        };
        if must_be_above {
            if price > self.market_price {
                AutocloseValidation::Valid
            } else {
                AutocloseValidation::TriggerMustBeHigher
            }
        } else if price < self.market_price {
            AutocloseValidation::Valid
        } else {
            AutocloseValidation::TriggerMustBeLower
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_amount() {
        let validator = AutocloseValidator::new(TpslType::TakeProfit, PerpetualDirection::Long, 100.0);
        assert_eq!(validator.validate(0.0), AutocloseValidation::InvalidAmount);
        assert_eq!(validator.validate(-1.0), AutocloseValidation::InvalidAmount);
    }

    #[test]
    fn test_long_take_profit_must_be_above_market() {
        let validator = AutocloseValidator::new(TpslType::TakeProfit, PerpetualDirection::Long, 100.0);
        assert_eq!(validator.validate(110.0), AutocloseValidation::Valid);
        assert_eq!(validator.validate(90.0), AutocloseValidation::TriggerMustBeHigher);
        assert_eq!(validator.validate(100.0), AutocloseValidation::TriggerMustBeHigher);
    }

    #[test]
    fn test_long_stop_loss_must_be_below_market() {
        let validator = AutocloseValidator::new(TpslType::StopLoss, PerpetualDirection::Long, 100.0);
        assert_eq!(validator.validate(90.0), AutocloseValidation::Valid);
        assert_eq!(validator.validate(110.0), AutocloseValidation::TriggerMustBeLower);
        assert_eq!(validator.validate(100.0), AutocloseValidation::TriggerMustBeLower);
    }

    #[test]
    fn test_short_take_profit_must_be_below_market() {
        let validator = AutocloseValidator::new(TpslType::TakeProfit, PerpetualDirection::Short, 100.0);
        assert_eq!(validator.validate(90.0), AutocloseValidation::Valid);
        assert_eq!(validator.validate(110.0), AutocloseValidation::TriggerMustBeLower);
    }

    #[test]
    fn test_short_stop_loss_must_be_above_market() {
        let validator = AutocloseValidator::new(TpslType::StopLoss, PerpetualDirection::Short, 100.0);
        assert_eq!(validator.validate(110.0), AutocloseValidation::Valid);
        assert_eq!(validator.validate(90.0), AutocloseValidation::TriggerMustBeHigher);
    }
}

#[derive(Debug, Clone)]
pub struct AutocloseEstimator {
    pub entry_price: f64,
    pub position_size: f64,
    pub direction: PerpetualDirection,
    pub leverage: u8,
}

impl AutocloseEstimator {
    pub fn new(entry_price: f64, position_size: f64, direction: PerpetualDirection, leverage: u8) -> Self {
        Self {
            entry_price,
            position_size,
            direction,
            leverage,
        }
    }

    pub fn for_open(market_price: f64, size: f64, leverage: u8, direction: PerpetualDirection) -> Self {
        let position_size = if market_price > 0.0 { size * f64::from(leverage) / market_price } else { 0.0 };
        Self::new(market_price, position_size, direction, leverage)
    }

    pub fn has_size(&self) -> bool {
        self.position_size != 0.0
    }

    pub fn pnl(&self, price: f64) -> f64 {
        let side = match self.direction {
            PerpetualDirection::Long => 1.0,
            PerpetualDirection::Short => -1.0,
        };
        side * (price - self.entry_price) * self.position_size.abs()
    }

    pub fn price_change_percent(&self, price: f64) -> f64 {
        let raw = crate::price_change::PriceChangeCalculator::percentage(self.entry_price, price);
        match self.direction {
            PerpetualDirection::Short => -raw,
            PerpetualDirection::Long => raw,
        }
    }

    pub fn roe(&self, price: f64) -> f64 {
        self.price_change_percent(price) * f64::from(self.leverage)
    }

    pub fn target_price_from_roe(&self, roe_percent: i32, trigger_type: TpslType) -> f64 {
        let leverage = f64::from(self.leverage.max(1));
        let fraction = f64::from(roe_percent) / leverage / 100.0;
        let sign = match (&self.direction, trigger_type) {
            (PerpetualDirection::Long, TpslType::TakeProfit) | (PerpetualDirection::Short, TpslType::StopLoss) => 1.0,
            (PerpetualDirection::Long, TpslType::StopLoss) | (PerpetualDirection::Short, TpslType::TakeProfit) => -1.0,
        };
        self.entry_price * (1.0 + sign * fraction)
    }
}

#[cfg(test)]
mod estimator_tests {
    use super::*;

    fn long(size: f64, leverage: u8) -> AutocloseEstimator {
        AutocloseEstimator::new(100.0, size, PerpetualDirection::Long, leverage)
    }

    #[test]
    fn test_has_size_counts_a_short_position_held_as_a_negative_size() {
        assert!(long(1.0, 2).has_size());
        assert!(AutocloseEstimator::new(100.0, -1.0, PerpetualDirection::Short, 2).has_size());
        assert!(!long(0.0, 2).has_size());
    }

    #[test]
    fn test_pnl_follows_the_direction_and_ignores_the_size_sign() {
        assert_eq!(long(2.0, 1).pnl(110.0), 20.0);
        assert_eq!(AutocloseEstimator::new(100.0, -2.0, PerpetualDirection::Short, 1).pnl(110.0), -20.0);
        assert_eq!(AutocloseEstimator::new(100.0, -2.0, PerpetualDirection::Short, 1).pnl(90.0), 20.0);
    }

    #[test]
    fn test_target_price_from_roe_survives_a_zero_leverage() {
        assert!((long(1.0, 2).target_price_from_roe(20, TpslType::TakeProfit) - 110.0).abs() < 1e-9);
        assert!((long(1.0, 2).target_price_from_roe(20, TpslType::StopLoss) - 90.0).abs() < 1e-9);
        assert!(long(1.0, 0).target_price_from_roe(20, TpslType::TakeProfit).is_finite());
    }
}
