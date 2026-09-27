use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::currency::Currency;
use primitives::{Asset, CustomFee, FeeUnitType};

use crate::config::fee_config::get_fee_config;
use crate::formatted_number::GemFormattedNumber;
use crate::models::custom_types::GemBigInt;
use crate::precision::GemValueStyle;
use crate::services::amount::model::GemNumberFormat;
use crate::services::amount::rules::{input_text, sanitize_number_input, value_from_input};
use crate::services::assets::model::GemFeeAmount;
use crate::services::assets::rules::fee_amount;
use crate::services::confirm::GemFeeRateRows;
use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCustomFeeCheck {
    Valid,
    BelowMinimum { rate: GemLocalizedText },
    OverMaximum { rate: GemLocalizedText },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeSession {
    pub fee_asset: Asset,
    pub input: String,
    pub format: GemNumberFormat,
    pub rows: GemFeeRateRows,
    pub loaded_fee: Option<GemBigInt>,
    pub price: Option<f64>,
    pub currency: Currency,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeEstimate {
    pub rate: Option<BigInt>,
    pub placeholder: Option<GemFormattedNumber>,
    pub fee_value: BigInt,
    pub fee: Option<GemFeeAmount>,
    pub check: GemCustomFeeCheck,
    pub is_valid: bool,
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

impl GemCustomFeeSession {
    pub(crate) fn new(fee_asset: Asset, format: GemNumberFormat, rows: GemFeeRateRows, loaded_fee: Option<GemBigInt>, price: Option<f64>, currency: Currency, rate: Option<&GemBigInt>) -> Self {
        let input = rate.and_then(|rate| input_text(&format.decimal_separator, &rate.to_string(), rows.unit_decimals)).unwrap_or_default();
        Self {
            fee_asset,
            input,
            format,
            rows,
            loaded_fee,
            price,
            currency,
        }
    }
}

#[uniffi::export]
impl GemCustomFeeSession {
    pub fn on_input(&self, text: String) -> Self {
        Self {
            input: sanitize_number_input(&self.format.decimal_separator, &text, Some(self.rows.unit_decimals), None),
            ..self.clone()
        }
    }

    pub fn view_state(&self) -> GemCustomFeeEstimate {
        let Self {
            fee_asset,
            input,
            format,
            rows,
            loaded_fee,
            price,
            currency,
        } = self;
        let config = get_fee_config(fee_asset.chain());
        let rate = value_from_input(&format.decimal_separator, input, rows.unit_decimals).ok().filter(|rate| rate > &BigInt::ZERO);
        let base_total = rows.selected_total.clone().unwrap_or_default();
        let normal_total = rows.normal_total.clone().unwrap_or_else(|| base_total.clone());
        let fee = CustomFee::calculate(
            rate.clone(),
            loaded_fee.clone().unwrap_or_default(),
            base_total,
            normal_total,
            config.max_multiplier,
            config.minimum_custom_fee_rate.map(BigInt::from),
        );
        let text = |rate: &BigInt| fee_rate_text(rows.unit_type, rate, rows.unit_decimals, &fee_asset.symbol);
        let check = match (&fee.minimum_rate, fee.is_below_minimum, fee.is_over_max) {
            (Some(minimum), true, _) => GemCustomFeeCheck::BelowMinimum { rate: text(minimum) },
            (_, _, true) => GemCustomFeeCheck::OverMaximum { rate: text(&fee.max_rate) },
            _ => GemCustomFeeCheck::Valid,
        };
        GemCustomFeeEstimate {
            placeholder: rows
                .selected_total
                .as_ref()
                .map(|total| GemFormattedNumber::amount(BigNumberFormatter::f64_value(total, rows.unit_decimals), None, GemValueStyle::Auto)),
            fee: loaded_fee.as_ref().map(|_| fee_amount(fee_asset, &fee.fee_value, *price, currency.clone())),
            fee_value: fee.fee_value,
            is_valid: fee.is_valid,
            check,
            rate,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::Chain;

    fn rows(selected_total: Option<u32>) -> GemFeeRateRows {
        GemFeeRateRows {
            rows: vec![],
            shows_options: true,
            unit_type: FeeUnitType::SatVb,
            unit_decimals: 1,
            selected_total: selected_total.map(BigInt::from),
            normal_total: None,
        }
    }

    fn session(chain: Chain, rows: GemFeeRateRows, loaded_fee: Option<BigInt>, rate: Option<&BigInt>) -> GemCustomFeeSession {
        GemCustomFeeSession::new(Asset::from_chain(chain), GemNumberFormat { decimal_separator: ".".to_string() }, rows, loaded_fee, Some(2.0), Currency::USD, rate)
    }

    fn estimate_with(chain: Chain, input: &str, rows: GemFeeRateRows, loaded_fee: Option<BigInt>) -> GemCustomFeeEstimate {
        GemCustomFeeSession {
            input: input.to_string(),
            ..session(chain, rows, loaded_fee, None)
        }
        .view_state()
    }

    fn estimate(chain: Chain, input: &str) -> GemCustomFeeEstimate {
        estimate_with(chain, input, rows(Some(100)), Some(BigInt::from(1_000)))
    }

    #[test]
    fn test_a_session_opens_on_the_picked_rate_and_keeps_typing_to_the_unit_decimals() {
        let picked = session(Chain::Bitcoin, rows(Some(100)), Some(BigInt::from(1_000)), Some(&BigInt::from(25)));
        assert_eq!(picked.input, "2.5", "a custom rate reopens as the text it was typed as");
        assert_eq!(session(Chain::Bitcoin, rows(Some(100)), None, None).input, "");

        let typed = picked.on_input("3.456".to_string());
        assert_eq!(typed.input, "3.4", "the field stops at the unit decimals");
        assert_eq!(typed.view_state().rate, Some(BigInt::from(34)));
    }

    #[test]
    fn test_the_typed_rate_is_read_in_the_fee_unit() {
        assert_eq!(estimate(Chain::Bitcoin, "2.5").rate, Some(BigInt::from(25)));
        assert_eq!(estimate(Chain::Bitcoin, "").rate, None);
        assert_eq!(estimate(Chain::Bitcoin, "0").rate, None, "a zero rate is no rate");
        assert!(!estimate(Chain::Bitcoin, "").is_valid);
        assert_eq!(estimate(Chain::Bitcoin, "").check, GemCustomFeeCheck::Valid, "an empty field shows no error");
        assert_eq!(estimate(Chain::Bitcoin, "").placeholder, Some(GemFormattedNumber::amount(10.0, None, GemValueStyle::Auto)));
        let unloaded = estimate_with(Chain::Bitcoin, "20", rows(None), Some(BigInt::from(1_000)));
        assert_eq!(unloaded.placeholder, None, "no loaded rate, no placeholder");
    }

    #[test]
    fn test_the_check_names_the_bound_a_custom_rate_breaks_with_its_rate() {
        assert_eq!(estimate(Chain::Bitcoin, "20").check, GemCustomFeeCheck::Valid);
        assert_eq!(
            estimate(Chain::BitcoinCash, "1").check,
            GemCustomFeeCheck::BelowMinimum {
                rate: fee_rate_text(FeeUnitType::SatVb, &BigInt::from(50), 1, "BCH")
            }
        );
        let over = estimate(Chain::Bitcoin, "100000");
        assert_eq!(
            over.check,
            GemCustomFeeCheck::OverMaximum {
                rate: fee_rate_text(FeeUnitType::SatVb, &BigInt::from(1_000), 1, "BTC")
            }
        );
        assert!(!over.is_valid);
    }

    #[test]
    fn test_the_fee_reads_in_the_fee_asset_once_a_fee_is_loaded() {
        let loaded = estimate(Chain::Bitcoin, "20");
        assert_eq!(loaded.fee, Some(fee_amount(&Asset::from_chain(Chain::Bitcoin), &loaded.fee_value, Some(2.0), Currency::USD)));
        assert_eq!(estimate_with(Chain::Bitcoin, "20", rows(Some(100)), None).fee, None, "no loaded fee, no amount to show");
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
