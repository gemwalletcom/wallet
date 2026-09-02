use crate::config::swap_config::get_swap_config;
use crate::models::custom_types::GemBigUint;
use number_formatter::BigNumberFormatter;
use std::sync::Arc;

pub use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData};
pub use swapper::SwapperProvider;

pub type GemApprovalData = ApprovalData;
pub type GemSwapData = SwapData;
pub type GemSwapQuoteData = SwapQuoteData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum GemSlippageCheck {
    Valid,
    High,
    BelowMinimum,
    AboveMaximum,
}

#[derive(Debug, Clone, PartialEq, uniffi::Object)]
pub struct GemSwapValue {
    value: GemBigUint,
    decimals: u32,
    price: Option<f64>,
}

#[uniffi::export]
impl GemSwapValue {
    #[uniffi::constructor]
    pub fn new(value: GemBigUint, decimals: u32, price: Option<f64>) -> Self {
        Self { value, decimals, price }
    }

    pub fn price_impact(&self, receive: Arc<GemSwapValue>) -> Option<SwapPriceImpact> {
        calculate_swap_price_impact(self.fiat_value()?, receive.fiat_value()?)
    }
}

impl GemSwapValue {
    fn fiat_value(&self) -> Option<f64> {
        let price = self.price?;
        let amount = BigNumberFormatter::value_as_f64(&self.value.to_string(), self.decimals).ok()?;
        Some(amount * price)
    }
}

pub fn calculate_swap_price_impact(pay_fiat_value: f64, receive_fiat_value: f64) -> Option<SwapPriceImpact> {
    if pay_fiat_value <= 0.0 || receive_fiat_value <= 0.0 || !pay_fiat_value.is_finite() || !receive_fiat_value.is_finite() {
        return None;
    }

    let percentage = ((receive_fiat_value / pay_fiat_value) - 1.0) * 100.0;
    let rounded_percentage = round_to_places(percentage, 2);
    let impact_type = match rounded_percentage {
        value if value > 0.0 => SwapPriceImpactType::Positive,
        value if value >= -1.0 => SwapPriceImpactType::Low,
        value if value >= -5.0 => SwapPriceImpactType::Medium,
        _ => SwapPriceImpactType::High,
    };

    Some(SwapPriceImpact {
        percentage,
        impact_type,
        is_high: rounded_percentage.abs() >= get_swap_config().high_price_impact_percent as f64,
    })
}

fn round_to_places(value: f64, places: i32) -> f64 {
    let factor = 10_f64.powi(places);
    (value * factor).round() / factor
}

#[cfg(test)]
mod tests {
    use super::{GemSwapValue, SwapPriceImpact, SwapPriceImpactType, calculate_swap_price_impact, round_to_places};
    use std::sync::Arc;

    #[test]
    fn test_swap_price_impact_needs_a_price_on_both_sides() {
        let priced = |value: u32, price: Option<f64>| Arc::new(GemSwapValue::new(value.into(), 2, price));

        assert_eq!(priced(100, None).price_impact(priced(100, Some(1.0))), None);
        assert_eq!(priced(100, Some(1.0)).price_impact(priced(100, None)), None);

        let impact = priced(200, Some(1.0)).price_impact(priced(100, Some(1.0))).expect("impact");
        assert_eq!(impact.percentage, -50.0);
    }

    #[test]
    fn test_calculate_swap_price_impact() {
        assert_eq!(calculate_swap_price_impact(0.0, 100.0), None);
        assert_eq!(calculate_swap_price_impact(100.0, 0.0), None);

        assert_eq!(
            calculate_swap_price_impact(100.0, 100.5).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: 0.5,
                impact_type: SwapPriceImpactType::Positive,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 99.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -1.0,
                impact_type: SwapPriceImpactType::Low,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 95.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -5.0,
                impact_type: SwapPriceImpactType::Medium,
                is_high: false,
            })
        );

        assert_eq!(
            calculate_swap_price_impact(100.0, 89.0).map(|impact| SwapPriceImpact {
                percentage: round_to_places(impact.percentage, 2),
                impact_type: impact.impact_type,
                is_high: impact.is_high,
            }),
            Some(SwapPriceImpact {
                percentage: -11.0,
                impact_type: SwapPriceImpactType::High,
                is_high: true,
            })
        );
    }
}
