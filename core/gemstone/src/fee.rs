use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::{Asset, Chain, CustomFee, FeeUnitType};

use crate::config::fee_config::get_fee_config;
use crate::formatted_number::GemFormattedNumber;
use crate::precision::GemValueStyle;
use crate::services::amount::model::GemNumberFormat;
use crate::services::amount::rules::value_from_input;
use crate::services::confirm::GemFeeRateRows;
use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCustomFeeCheck {
    Valid,
    BelowMinimum { rate: GemLocalizedText },
    OverMaximum { rate: GemLocalizedText },
}

#[derive(uniffi::Object, Clone, Debug, PartialEq)]
pub struct GemCustomFee {
    fee: CustomFee,
    rate: Option<BigInt>,
    base_total: Option<BigInt>,
    unit_type: FeeUnitType,
    unit_decimals: u32,
    symbol: String,
}

pub fn fee_rate_text(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str) -> GemLocalizedText {
    let value = BigNumberFormatter::f64_value(rate, decimals);
    let rate = match unit_type {
        FeeUnitType::Gwei => GemFormattedNumber::adaptive(value, None),
        FeeUnitType::SatVb => GemFormattedNumber::amount(value, None, GemValueStyle::Full),
        FeeUnitType::Native => GemFormattedNumber::amount(value, Some(symbol.to_string()), GemValueStyle::Full),
    };
    GemLocalizedText::FeeRate { rate, unit: unit_type }
}

#[uniffi::export]
impl GemCustomFee {
    #[uniffi::constructor]
    pub fn estimate(chain: Chain, input: String, format: GemNumberFormat, rows: GemFeeRateRows, loaded_fee: BigInt) -> Self {
        let config = get_fee_config(chain);
        let rate = value_from_input(&format.decimal_separator, &input, rows.unit_decimals).ok().filter(|rate| rate > &BigInt::ZERO);
        let base_total = rows.selected_total.clone().unwrap_or_default();
        let normal_total = rows.normal_total.unwrap_or_else(|| base_total.clone());
        Self {
            fee: CustomFee::calculate(rate.clone(), loaded_fee, base_total, normal_total, config.max_multiplier, config.minimum_custom_fee_rate.map(BigInt::from)),
            rate,
            base_total: rows.selected_total,
            unit_type: rows.unit_type,
            unit_decimals: rows.unit_decimals,
            symbol: Asset::from_chain(chain).symbol,
        }
    }

    pub fn rate(&self) -> Option<BigInt> {
        self.rate.clone()
    }

    pub fn fee_value(&self) -> BigInt {
        self.fee.fee_value.clone()
    }

    pub fn placeholder(&self) -> Option<GemFormattedNumber> {
        let total = self.base_total.as_ref()?;
        let value = BigNumberFormatter::f64_value(total, self.unit_decimals);
        Some(GemFormattedNumber::amount(value, None, GemValueStyle::Auto))
    }

    pub fn check(&self) -> GemCustomFeeCheck {
        let text = |rate: &BigInt| fee_rate_text(self.unit_type, rate, self.unit_decimals, &self.symbol);
        match (&self.fee.minimum_rate, self.fee.is_below_minimum, self.fee.is_over_max) {
            (Some(minimum), true, _) => GemCustomFeeCheck::BelowMinimum { rate: text(minimum) },
            (_, _, true) => GemCustomFeeCheck::OverMaximum { rate: text(&self.fee.max_rate) },
            _ => GemCustomFeeCheck::Valid,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.fee.is_valid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(selected_total: Option<u32>) -> GemFeeRateRows {
        GemFeeRateRows {
            rows: vec![],
            shows_options: true,
            unit_type: FeeUnitType::SatVb,
            unit_decimals: 1,
            supports_custom_fee: true,
            selected_total: selected_total.map(BigInt::from),
            normal_total: None,
            custom_rate: None,
        }
    }

    fn estimate(chain: Chain, input: &str) -> GemCustomFee {
        GemCustomFee::estimate(chain, input.to_string(), GemNumberFormat { decimal_separator: ".".to_string() }, rows(Some(100)), BigInt::from(1_000))
    }

    #[test]
    fn test_the_typed_rate_is_read_in_the_fee_unit() {
        assert_eq!(estimate(Chain::Bitcoin, "2.5").rate(), Some(BigInt::from(25)));
        assert_eq!(estimate(Chain::Bitcoin, "").rate(), None);
        assert_eq!(estimate(Chain::Bitcoin, "0").rate(), None, "a zero rate is no rate");
        assert!(!estimate(Chain::Bitcoin, "").is_valid());
        assert_eq!(estimate(Chain::Bitcoin, "").check(), GemCustomFeeCheck::Valid, "an empty field shows no error");
        assert_eq!(estimate(Chain::Bitcoin, "").placeholder(), Some(GemFormattedNumber::amount(10.0, None, GemValueStyle::Auto)));
        let unloaded = GemCustomFee::estimate(Chain::Bitcoin, "20".to_string(), GemNumberFormat { decimal_separator: ".".to_string() }, rows(None), BigInt::from(1_000));
        assert_eq!(unloaded.placeholder(), None, "no loaded rate, no placeholder");
    }

    #[test]
    fn test_the_check_names_the_bound_a_custom_rate_breaks_with_its_rate() {
        assert_eq!(estimate(Chain::Bitcoin, "20").check(), GemCustomFeeCheck::Valid);
        assert_eq!(
            estimate(Chain::BitcoinCash, "1").check(),
            GemCustomFeeCheck::BelowMinimum {
                rate: fee_rate_text(FeeUnitType::SatVb, &BigInt::from(50), 1, "BCH")
            }
        );
        let over = estimate(Chain::Bitcoin, "100000");
        assert_eq!(
            over.check(),
            GemCustomFeeCheck::OverMaximum {
                rate: fee_rate_text(FeeUnitType::SatVb, &BigInt::from(1_000), 1, "BTC")
            }
        );
        assert!(!over.is_valid());
    }

    #[test]
    fn test_a_fee_rate_reads_in_its_unit() {
        assert_eq!(
            fee_rate_text(FeeUnitType::SatVb, &BigInt::from(1_000_000), 1, "BTC"),
            GemLocalizedText::FeeRate {
                rate: GemFormattedNumber::amount(100_000.0, None, GemValueStyle::Full),
                unit: FeeUnitType::SatVb
            }
        );
        assert_eq!(
            fee_rate_text(FeeUnitType::Gwei, &BigInt::from(123_456_789), 9, "ETH"),
            GemLocalizedText::FeeRate {
                rate: GemFormattedNumber::adaptive(0.123456789, None),
                unit: FeeUnitType::Gwei
            },
            "gwei keeps the significant digits of a small rate"
        );
        assert_eq!(
            fee_rate_text(FeeUnitType::Native, &BigInt::from(2_500), 9, "SOL"),
            GemLocalizedText::FeeRate {
                rate: GemFormattedNumber::amount(0.0000025, Some("SOL".to_string()), GemValueStyle::Full),
                unit: FeeUnitType::Native
            },
            "a native fee reads in the fee asset"
        );
    }
}
