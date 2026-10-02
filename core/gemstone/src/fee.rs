use num_bigint::BigInt;
use number_formatter::BigNumberFormatter;
use primitives::currency::Currency;
use primitives::{Asset, FeeUnitType, GasPriceType, custom_fee_value};

use crate::config::fee_config::get_fee_config;
use crate::formatted_number::{GemFormattedNumber, GemNumberRounding, GemNumberUnit};
use crate::models::custom_types::GemBigInt;
use crate::models::list::GemListRowTitle;
use crate::precision::GemValueStyle;
use crate::services::amount::model::GemNumberFormat;
use crate::services::amount::rules::{input_text, sanitize_number_input, value_from_input};
use crate::services::assets::model::GemFeeAmount;
use crate::services::assets::rules::fee_amount;
use crate::services::confirm::{GemConfirmFeeSelection, GemFeeRateRows};
use crate::services::localization::GemLocalizedText;

#[derive(Debug, Clone, PartialEq, uniffi::Enum)]
pub enum GemCustomFeeCheck {
    Valid,
    BelowMinimum { rate: GemLocalizedText },
    OverMaximum { rate: GemLocalizedText },
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeField {
    pub title: GemListRowTitle,
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeFieldState {
    pub placeholder: Option<GemFormattedNumber>,
    pub check: GemCustomFeeCheck,
    pub hint_title: GemLocalizedText,
    pub hint: Option<GemLocalizedText>,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeSession {
    pub fee_asset: Asset,
    pub format: GemNumberFormat,
    pub rows: GemFeeRateRows,
    pub loaded_fee: Option<GemBigInt>,
    pub price: Option<f64>,
    pub currency: Currency,
    pub base_fee: Option<GemCustomFeeField>,
    pub rate: GemCustomFeeField,
}

#[derive(Debug, Clone, PartialEq, uniffi::Record)]
pub struct GemCustomFeeEstimate {
    pub selection: Option<GemConfirmFeeSelection>,
    pub base_fee: Option<GemCustomFeeFieldState>,
    pub rate: GemCustomFeeFieldState,
    pub fee_value: BigInt,
    pub fee: Option<GemFeeAmount>,
}

pub fn fee_rate_text(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str) -> GemLocalizedText {
    GemLocalizedText::FeeRate {
        rate: fee_rate_number(unit_type, rate, decimals, symbol),
        unit: unit_type,
    }
}

fn fee_rate_number(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str) -> GemFormattedNumber {
    let value = BigNumberFormatter::f64_value(rate, decimals);
    match unit_type {
        FeeUnitType::Gwei => GemFormattedNumber::adaptive(value, None),
        FeeUnitType::SatVb => GemFormattedNumber::amount(value, None, GemValueStyle::Full),
        FeeUnitType::Native => GemFormattedNumber::amount(value, Some(symbol.to_string()), GemValueStyle::Full),
    }
}

pub(crate) fn rate_of(gas_price: &GasPriceType) -> BigInt {
    match gas_price {
        GasPriceType::Regular { gas_price } | GasPriceType::Solana { gas_price, .. } => gas_price.clone(),
        GasPriceType::Eip1559 { priority_fee, .. } => priority_fee.clone(),
    }
}

impl GemCustomFeeSession {
    pub(crate) fn new(fee_asset: Asset, format: GemNumberFormat, rows: GemFeeRateRows, loaded_fee: Option<GemBigInt>, price: Option<f64>, currency: Currency, selection: Option<&GemConfirmFeeSelection>) -> Self {
        let (typed_base_fee, typed_rate) = match selection {
            Some(GemConfirmFeeSelection::Custom { base_fee, rate }) => (base_fee.clone(), rate.clone()),
            Some(GemConfirmFeeSelection::Priority { .. }) | None => (None, None),
        };
        let input = |value: Option<BigInt>| value.and_then(|value| input_text(&format.decimal_separator, &value.to_string(), rows.unit_decimals)).unwrap_or_default();
        let base_fee = rows.base_fee.as_ref().filter(|network_base_fee| *network_base_fee > &BigInt::ZERO).map(|_| GemCustomFeeField {
            title: GemListRowTitle::BaseFee,
            input: input(typed_base_fee),
        });
        let rate = GemCustomFeeField {
            title: match rows.base_fee {
                Some(_) => GemListRowTitle::PriorityFee,
                None => GemListRowTitle::CustomFee,
            },
            input: input(typed_rate),
        };
        Self {
            fee_asset,
            format,
            rows,
            loaded_fee,
            price,
            currency,
            base_fee,
            rate,
        }
    }

    fn sanitized(&self, text: &str) -> String {
        sanitize_number_input(&self.format.decimal_separator, text, Some(self.rows.unit_decimals), None)
    }

    fn read(&self, field: &GemCustomFeeField, minimum: BigInt, maximum: BigInt, hint_title: GemLocalizedText, hint: Option<BigInt>) -> FieldReading {
        let rows = &self.rows;
        let number = |value: &BigInt, rounding| GemFormattedNumber {
            rounding,
            ..fee_rate_number(rows.unit_type, value, rows.unit_decimals, &self.fee_asset.symbol)
        };
        let text = |value: &BigInt, rounding| GemLocalizedText::FeeRate {
            rate: number(value, rounding),
            unit: rows.unit_type,
        };
        let typed = value_from_input(&self.format.decimal_separator, &field.input, rows.unit_decimals).ok();
        let value = typed.clone().or(hint.clone()).unwrap_or_default();
        let check = if value < minimum {
            GemCustomFeeCheck::BelowMinimum {
                rate: text(&minimum, GemNumberRounding::AwayFromZero),
            }
        } else if value > maximum {
            GemCustomFeeCheck::OverMaximum {
                rate: text(&maximum, GemNumberRounding::TowardZero),
            }
        } else {
            GemCustomFeeCheck::Valid
        };
        FieldReading {
            typed,
            value,
            valid: check == GemCustomFeeCheck::Valid,
            state: GemCustomFeeFieldState {
                placeholder: hint.as_ref().map(|value| GemFormattedNumber {
                    unit: GemNumberUnit::Plain,
                    ..number(value, GemNumberRounding::AwayFromZero)
                }),
                hint_title,
                hint: hint.as_ref().map(|value| text(value, GemNumberRounding::AwayFromZero)),
                check,
            },
        }
    }
}

struct FieldReading {
    typed: Option<BigInt>,
    value: BigInt,
    valid: bool,
    state: GemCustomFeeFieldState,
}

#[uniffi::export]
impl GemCustomFeeSession {
    pub fn on_input(&self, text: String) -> Self {
        Self {
            rate: GemCustomFeeField {
                input: self.sanitized(&text),
                ..self.rate.clone()
            },
            ..self.clone()
        }
    }

    pub fn on_base_fee_input(&self, text: String) -> Self {
        Self {
            base_fee: self.base_fee.as_ref().map(|field| GemCustomFeeField {
                input: self.sanitized(&text),
                ..field.clone()
            }),
            ..self.clone()
        }
    }

    pub fn view_state(&self) -> GemCustomFeeEstimate {
        let config = get_fee_config(self.fee_asset.chain());
        let rows = &self.rows;
        let maximum = rows.normal.as_ref().map(GasPriceType::total_fee).unwrap_or_default() * config.max_multiplier;
        let rate = self.read(
            &self.rate,
            config.minimum_custom_fee_rate.unwrap_or_default(),
            maximum.clone(),
            GemLocalizedText::SuggestedFeeRate,
            rows.normal.as_ref().map(rate_of),
        );
        let base_fee = self
            .base_fee
            .as_ref()
            .zip(rows.base_fee.as_ref())
            .map(|(field, network_base_fee)| self.read(field, network_base_fee.clone(), maximum, GemLocalizedText::CurrentBaseFee, Some(network_base_fee.clone())));
        let base = base_fee.as_ref().map(|base_fee| base_fee.value.clone()).or(rows.base_fee.clone());
        let gas_price = match &base {
            Some(base) => GasPriceType::eip1559(base.clone(), rate.value.clone()),
            None => GasPriceType::regular(rate.value.clone()),
        };
        let valid = rate.valid && base_fee.as_ref().is_none_or(|base_fee| base_fee.valid);
        let loaded_fee = self.loaded_fee.clone().unwrap_or_default();
        let fee_value = custom_fee_value(&loaded_fee, &rows.selected.as_ref().map(GasPriceType::total_fee).unwrap_or_default(), &gas_price.total_fee());
        GemCustomFeeEstimate {
            selection: valid.then(|| GemConfirmFeeSelection::Custom {
                base_fee: base_fee.as_ref().and_then(|base_fee| base_fee.typed.clone()),
                rate: rate.typed.clone(),
            }),
            base_fee: base_fee.map(|base_fee| base_fee.state),
            rate: rate.state,
            fee: self.loaded_fee.as_ref().map(|_| fee_amount(&self.fee_asset, &fee_value, self.price, self.currency.clone())),
            fee_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::confirm::testkit::gwei;
    use primitives::Chain;

    fn rows(selected: Option<u32>) -> GemFeeRateRows {
        GemFeeRateRows {
            rows: vec![],
            shows_options: true,
            unit_type: FeeUnitType::SatVb,
            unit_decimals: 1,
            selected: selected.map(GasPriceType::regular),
            normal: selected.map(GasPriceType::regular),
            base_fee: None,
        }
    }

    fn evm_rows() -> GemFeeRateRows {
        GemFeeRateRows {
            rows: vec![],
            shows_options: true,
            unit_type: FeeUnitType::Gwei,
            unit_decimals: 9,
            selected: Some(GasPriceType::eip1559(gwei(24), gwei(1))),
            normal: Some(GasPriceType::eip1559(gwei(24), gwei(1))),
            base_fee: Some(gwei(20)),
        }
    }

    fn custom(base_fee: Option<BigInt>, rate: Option<BigInt>) -> GemConfirmFeeSelection {
        GemConfirmFeeSelection::Custom { base_fee, rate }
    }

    fn session(chain: Chain, rows: GemFeeRateRows, loaded_fee: Option<BigInt>, custom: Option<&GemConfirmFeeSelection>) -> GemCustomFeeSession {
        GemCustomFeeSession::new(Asset::from_chain(chain), GemNumberFormat { decimal_separator: ".".to_string() }, rows, loaded_fee, Some(2.0), Currency::USD, custom)
    }

    fn estimate_with(chain: Chain, input: &str, rows: GemFeeRateRows, loaded_fee: Option<BigInt>) -> GemCustomFeeEstimate {
        session(chain, rows, loaded_fee, None).on_input(input.to_string()).view_state()
    }

    fn estimate(chain: Chain, input: &str) -> GemCustomFeeEstimate {
        estimate_with(chain, input, rows(Some(100)), Some(BigInt::from(1_000)))
    }

    fn bound_text(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str, rounding: GemNumberRounding) -> GemLocalizedText {
        GemLocalizedText::FeeRate {
            rate: GemFormattedNumber {
                rounding,
                ..fee_rate_number(unit_type, rate, decimals, symbol)
            },
            unit: unit_type,
        }
    }

    fn minimum_text(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str) -> GemLocalizedText {
        bound_text(unit_type, rate, decimals, symbol, GemNumberRounding::AwayFromZero)
    }

    fn maximum_text(unit_type: FeeUnitType, rate: &BigInt, decimals: u32, symbol: &str) -> GemLocalizedText {
        bound_text(unit_type, rate, decimals, symbol, GemNumberRounding::TowardZero)
    }

    #[test]
    fn test_a_session_opens_on_the_picked_rate_and_keeps_typing_to_the_unit_decimals() {
        let picked = session(Chain::Bitcoin, rows(Some(100)), Some(BigInt::from(1_000)), Some(&custom(None, Some(BigInt::from(25)))));
        assert_eq!(picked.rate.input, "2.5", "a custom rate reopens as the text it was typed as");
        assert_eq!((picked.rate.title, picked.base_fee.clone()), (GemListRowTitle::CustomFee, None), "a rate with no base fee is one custom field");
        assert_eq!(session(Chain::Bitcoin, rows(Some(100)), None, None).rate.input, "");

        let typed = picked.on_input("3.456".to_string());
        assert_eq!(typed.rate.input, "3.4", "the field stops at the unit decimals");
        assert_eq!(typed.view_state().selection, Some(custom(None, Some(BigInt::from(34)))));
    }

    #[test]
    fn test_the_typed_rate_is_read_in_the_fee_unit() {
        assert_eq!(estimate(Chain::Bitcoin, "2.5").selection, Some(custom(None, Some(BigInt::from(25)))));
        assert_eq!(estimate(Chain::Bitcoin, "").selection, Some(custom(None, None)), "an empty field sends nothing typed, so the suggested rate is used");
        assert_eq!(estimate(Chain::Bitcoin, "0").selection, None, "a zero rate is under the chain minimum");
        assert_eq!(
            estimate(Chain::Bitcoin, "").rate.placeholder,
            Some(GemFormattedNumber {
                rounding: GemNumberRounding::AwayFromZero,
                ..GemFormattedNumber::amount(10.0, None, GemValueStyle::Full)
            })
        );
        let unloaded = estimate_with(Chain::Bitcoin, "20", rows(None), Some(BigInt::from(1_000)));
        assert_eq!(unloaded.rate.placeholder, None, "no loaded rate, no placeholder");
    }

    #[test]
    fn test_the_check_names_the_bound_a_custom_rate_breaks_rounded_so_the_shown_value_passes() {
        assert_eq!(estimate(Chain::Bitcoin, "20").rate.check, GemCustomFeeCheck::Valid);
        assert!(
            session(Chain::ZkSync, evm_rows(), Some(gwei(21_000)), None).on_input("0".to_string()).view_state().selection.is_some(),
            "a zero minimum takes zero"
        );
        assert_eq!(
            estimate(Chain::BitcoinCash, "1").rate.check,
            GemCustomFeeCheck::BelowMinimum {
                rate: minimum_text(FeeUnitType::SatVb, &BigInt::from(50), 1, "BCH")
            }
        );
        let over = estimate(Chain::Bitcoin, "100000");
        assert_eq!(
            over.rate.check,
            GemCustomFeeCheck::OverMaximum {
                rate: maximum_text(FeeUnitType::SatVb, &BigInt::from(1_000), 1, "BTC")
            }
        );
        assert_eq!(over.selection, None);
    }

    #[test]
    fn test_an_evm_session_takes_a_base_fee_and_a_tip() {
        let opened = session(Chain::Ethereum, evm_rows(), Some(gwei(25 * 21_000)), Some(&custom(Some(gwei(30)), Some(gwei(2)))));
        let base_fee = opened.base_fee.clone().unwrap();
        assert_eq!((base_fee.title, base_fee.input.as_str()), (GemListRowTitle::BaseFee, "30"));
        assert_eq!((opened.rate.title, opened.rate.input.as_str()), (GemListRowTitle::PriorityFee, "2"));

        let fresh = session(Chain::Ethereum, evm_rows(), Some(gwei(25 * 21_000)), None);
        assert_eq!((fresh.base_fee.as_ref().unwrap().input.as_str(), fresh.rate.input.as_str()), ("", ""), "both fields open empty");
        let estimate = fresh.view_state();
        assert_eq!(
            estimate.base_fee.as_ref().unwrap().hint,
            Some(minimum_text(FeeUnitType::Gwei, &gwei(20), 9, "ETH")),
            "the base fee field hints the network's base fee, rounded up like its minimum so typing what it shows passes"
        );
        assert_eq!(
            estimate.rate.placeholder,
            Some(GemFormattedNumber {
                rounding: GemNumberRounding::AwayFromZero,
                ..GemFormattedNumber::adaptive(1.0, None)
            }),
            "the tip field hints the normal tip"
        );
        assert_eq!(estimate.selection, Some(custom(None, None)), "an untouched screen keeps following the network");
        assert_eq!(estimate.fee_value, gwei(21 * 21_000), "the preview takes the hinted base fee and tip");

        let typed = fresh.on_base_fee_input("22".to_string()).on_input("5".to_string()).view_state();
        assert_eq!(typed.selection, Some(custom(Some(gwei(22)), Some(gwei(5)))));
        assert_eq!(typed.fee_value, gwei(27 * 21_000), "the fee follows the typed base fee plus the typed tip");

        let tip_only = fresh.on_input("5".to_string()).view_state();
        assert_eq!(tip_only.selection, Some(custom(None, Some(gwei(5)))), "a tip alone leaves the base fee to the network");
    }

    #[test]
    fn test_a_zero_base_fee_network_takes_only_a_tip() {
        let rows = GemFeeRateRows {
            selected: Some(GasPriceType::eip1559(BigInt::ZERO, gwei(1))),
            normal: Some(GasPriceType::eip1559(BigInt::ZERO, gwei(1))),
            base_fee: Some(BigInt::ZERO),
            ..evm_rows()
        };
        let opened = session(Chain::SmartChain, rows, Some(gwei(21_000)), None);
        assert_eq!(opened.base_fee, None, "a network without a base fee has no base fee to set");
        assert_eq!(opened.rate.title, GemListRowTitle::PriorityFee);

        let typed = opened.on_input("2".to_string()).view_state();
        assert_eq!(typed.selection, Some(custom(None, Some(gwei(2)))), "a network without a base fee takes only the tip");
        assert_eq!(typed.fee_value, gwei(2 * 21_000));
    }

    #[test]
    fn test_an_evm_base_fee_cannot_go_under_the_network_and_a_tip_under_the_chain_minimum() {
        let fresh = session(Chain::Ethereum, evm_rows(), Some(gwei(25 * 21_000)), None);
        let low_base = fresh.on_base_fee_input("19".to_string()).on_input("1".to_string()).view_state();
        assert_eq!(
            low_base.base_fee.unwrap().check,
            GemCustomFeeCheck::BelowMinimum {
                rate: minimum_text(FeeUnitType::Gwei, &gwei(20), 9, "ETH")
            },
            "the base fee floor is the network's current base fee, not the margin the wallet adds"
        );
        assert_eq!(low_base.selection, None);
        let at_network = fresh.on_base_fee_input("20".to_string()).on_input("1".to_string()).view_state();
        assert_eq!(at_network.base_fee.unwrap().check, GemCustomFeeCheck::Valid, "the network base fee itself is accepted");
        assert!(at_network.selection.is_some());

        let low_tip = fresh.on_base_fee_input("24".to_string()).on_input("0.005".to_string()).view_state();
        assert_eq!(
            low_tip.rate.check,
            GemCustomFeeCheck::BelowMinimum {
                rate: minimum_text(FeeUnitType::Gwei, &BigInt::from(10_000_000), 9, "ETH")
            }
        );
        assert_eq!(low_tip.selection, None);
    }

    #[test]
    fn test_each_evm_field_is_capped_at_a_multiple_of_the_normal_price() {
        let fresh = session(Chain::Ethereum, evm_rows(), Some(gwei(25 * 21_000)), None);
        let high_base = fresh.on_base_fee_input("251".to_string()).on_input("1".to_string()).view_state();
        assert_eq!(
            high_base.base_fee.unwrap().check,
            GemCustomFeeCheck::OverMaximum {
                rate: maximum_text(FeeUnitType::Gwei, &gwei(250), 9, "ETH")
            }
        );
        let high_tip = fresh.on_base_fee_input("24".to_string()).on_input("251".to_string()).view_state();
        assert_eq!(
            high_tip.rate.check,
            GemCustomFeeCheck::OverMaximum {
                rate: maximum_text(FeeUnitType::Gwei, &gwei(250), 9, "ETH")
            },
            "a tip is capped by the normal price as a whole, not by the normal tip, which can be tiny"
        );
        assert_eq!(high_tip.selection, None);
        assert!(fresh.on_base_fee_input("250".to_string()).on_input("250".to_string()).view_state().selection.is_some(), "the maximum itself is accepted");
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
