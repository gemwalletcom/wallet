use crate::config::swap_config::get_swap_config;
use primitives::swap::SwapQuoteDataType;

pub use primitives::swap::{ApprovalData, SwapData, SwapPriceImpact, SwapPriceImpactType, SwapProviderData, SwapQuote, SwapQuoteData};
pub use swapper::SwapperProvider;

pub type GemApprovalData = ApprovalData;
pub type GemSwapData = SwapData;
pub type GemSwapPriceImpact = SwapPriceImpact;
pub type GemSwapPriceImpactType = SwapPriceImpactType;
pub type GemSwapProviderData = SwapProviderData;
pub type GemSwapQuote = SwapQuote;
pub type GemSwapQuoteData = SwapQuoteData;
pub type GemSwapQuoteDataType = SwapQuoteDataType;

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
    use super::{SwapPriceImpact, SwapPriceImpactType, calculate_swap_price_impact, round_to_places};

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
