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
